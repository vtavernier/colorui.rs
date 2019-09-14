use actix::prelude::*;
use actix_files as fs;
use actix_web::{middleware::Logger, web, App, Error, HttpRequest, HttpResponse, HttpServer};
use actix_web_actors::ws;

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use failure::Fail;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[macro_use]
extern crate log;

struct State {
    port: Option<Box<dyn serialport::SerialPort>>,
}

#[derive(Debug, Fail)]
pub enum SendError {
    #[fail(display = "serial port error: {}", 0)]
    SerialError(serialport::Error),
    #[fail(display = "json error: {}", 0)]
    JsonError(serde_json::Error),
    #[fail(display = "i/o error: {}", 0)]
    IoError(std::io::Error),
}

impl State {
    fn new() -> Self {
        Self { port: None }
    }

    fn port(&mut self) -> Result<&mut dyn serialport::SerialPort, serialport::Error> {
        if !self.port.is_some() {
            if let Err(error) = serialport::available_ports().map(|ports| {
                ports
                    .into_iter()
                    .find(|port| {
                        if let serialport::SerialPortType::UsbPort(info) = &port.port_type {
                            info.vid == 0x0483
                        } else {
                            false
                        }
                    })
                    .map(|port| port.port_name)
                    .map(|port_name| {
                        match serialport::open_with_settings(
                            &port_name,
                            &serialport::SerialPortSettings {
                                baud_rate: 115200,
                                ..Default::default()
                            },
                        ) {
                            Ok(port) => {
                                self.port = Some(port);
                                Ok(())
                            }
                            Err(error) => Err(error),
                        }
                    })
            }) {
                return Err(error);
            }
        }

        Ok(&mut **self.port.as_mut().unwrap())
    }

    fn to_u8(value: f32) -> u8 {
        if value > 1.0 {
            return 255;
        }
        if value < 0.0 {
            return 0;
        }
        (255.0 * value) as u8
    }

    fn hsv_to_rgbw(hsv: palette::Hsv) -> (u8, u8, u8, u8) {
        let (h, s, v) = hsv.into_components();
        let (r, g, b) = palette::LinSrgb::from(palette::Hsv::new(h, 1., 1.)).into_components();

        let s = s.sqrt();
        let v = v * v;

        (
            Self::to_u8(r * s * v),
            Self::to_u8(g * s * v),
            Self::to_u8(b * s * v),
            Self::to_u8(v * (1. - s)),
        )
    }

    fn send(&mut self, req: CURequest) -> Result<(), SendError> {
        match req {
            CURequest::Led { led, r, g, b } => {
                // Convert input to RGBW
                let rgb = palette::LinSrgb::from_components((
                    r as f32 / 255.0,
                    g as f32 / 255.0,
                    b as f32 / 255.0,
                ));

                let (r, g, b, w) = Self::hsv_to_rgbw(rgb.into());

                // Forward request to serial port
                self.port()
                    .map_err(|error| SendError::SerialError(error))
                    .and_then(|port| {
                        let msg = json!({
                            "led": led,
                            "r": r,
                            "g": g,
                            "b": b,
                            "w": w,
                        });
                        info!("sending command: {:?}", msg);

                        // We have an open port
                        serde_json::to_writer(port, &msg).map_err(|error| {
                            if error.is_io() {
                                return SendError::IoError(error.into());
                            }

                            SendError::JsonError(error)
                        })
                    })
                    .map_err(|error| {
                        if let SendError::IoError(_io) = &error {
                            // Drop serial port, it failed
                            self.port = None
                        }

                        error
                    })
            }
        }
    }
}

type StateHandle = Mutex<State>;

/// How often heartbeat pings are sent
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
/// How long before lack of client response causes a timeout
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

/// do websocket handshake and start `WebSocketSession` actor
fn ws_index(
    s: web::Data<StateHandle>,
    r: HttpRequest,
    stream: web::Payload,
) -> Result<HttpResponse, Error> {
    ws::start(WebSocketSession::new(s.into_inner().clone()), &r, stream)
}

/// websocket connection is long running connection, it easier
/// to handle with an actor
struct WebSocketSession {
    /// Client must send ping at least once per 10 seconds (CLIENT_TIMEOUT),
    /// otherwise we drop connection.
    hb: Instant,
    /// State handle
    state: Arc<StateHandle>,
}

impl Actor for WebSocketSession {
    type Context = ws::WebsocketContext<Self>;

    /// Method is called on actor start. We start the heartbeat process here.
    fn started(&mut self, ctx: &mut Self::Context) {
        self.hb(ctx);
    }
}

#[derive(Debug, Deserialize)]
#[serde(tag = "method", rename_all = "snake_case")]
pub enum CURequest {
    Led { led: u8, r: u8, g: u8, b: u8 },
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum CUResponse {
    Success { success: bool },
    Error { success: bool, error: String },
}

impl CUResponse {
    fn success() -> Self {
        CUResponse::Success { success: true }
    }

    fn error(error: impl Into<String>) -> Self {
        CUResponse::Error {
            success: false,
            error: error.into(),
        }
    }
}

impl<E: ToString> From<Result<(), E>> for CUResponse {
    fn from(result: Result<(), E>) -> Self {
        match result {
            Ok(()) => CUResponse::success(),
            Err(error) => CUResponse::error(error.to_string()),
        }
    }
}

/// Handler for `ws::Message`
impl StreamHandler<ws::Message, ws::ProtocolError> for WebSocketSession {
    fn handle(&mut self, msg: ws::Message, ctx: &mut Self::Context) {
        // process websocket messages
        trace!("WS: {:?}", msg);
        match msg {
            ws::Message::Ping(msg) => {
                self.hb = Instant::now();
                ctx.pong(&msg);
            }
            ws::Message::Pong(_) => {
                self.hb = Instant::now();
            }
            ws::Message::Text(text) => {
                let res = CUResponse::from(
                    serde_json::from_str(&text)
                        .map_err(|err| SendError::JsonError(err))
                        .and_then(|req| self.state.lock().unwrap().send(req)),
                );

                info!("res: {:?}", res);
                ctx.text(serde_json::to_string(&res).unwrap());
            }
            ws::Message::Binary(_) => ctx
                .text(serde_json::to_string(&CUResponse::error("unexpected binary data")).unwrap()),
            ws::Message::Close(_) => {
                ctx.stop();
            }
            ws::Message::Nop => (),
        }
    }
}

impl WebSocketSession {
    fn new(state: Arc<StateHandle>) -> Self {
        Self {
            hb: Instant::now(),
            state,
        }
    }

    /// helper method that sends ping to client every second.
    ///
    /// also this method checks heartbeats from client
    fn hb(&self, ctx: &mut <Self as Actor>::Context) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            // check client heartbeats
            if Instant::now().duration_since(act.hb) > CLIENT_TIMEOUT {
                // heartbeat timed out
                info!("Websocket Client heartbeat failed, disconnecting!");

                // stop actor
                ctx.stop();

                // don't try to send a ping
                return;
            }

            ctx.ping("");
        });
    }
}

fn main() -> std::io::Result<()> {
    env_logger::Builder::from_env(env_logger::Env::new().default_filter_or("info")).init();

    let state = web::Data::new(Mutex::new(State::new()));

    HttpServer::new(move || {
        App::new()
            .register_data(state.clone())
            .wrap(Logger::default())
            .service(web::resource("/ws/").route(web::get().to(ws_index)))
            .service(fs::Files::new("/", "./web/dist").index_file("index.html"))
    })
    .bind("0.0.0.0:8000")?
    .run()
}
