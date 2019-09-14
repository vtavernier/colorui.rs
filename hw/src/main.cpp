#include <Arduino.h>
#include <USBSerial.h>

#include <ArduinoJson.h>
#include <stdint.h>

#define LED1W PA0
#define LED2W PA1
#define LED3W PA2

#define LED1B PA3
#define LED2B PA6
#define LED3B PA7

#define LED1G PB0
#define LED2G PB7
#define LED3G PB6

#define LED1R PA10
#define LED2R PA9
#define LED3R PA8

// clang-format off
const uint8_t LED_PINS[] = {
	LED1R, LED1G, LED1B, LED1W,
	LED2R, LED2G, LED2B, LED2W,
	LED3R, LED3G, LED3B, LED3W,
};
// clang-format on

struct CRGBW {
	uint8_t r;
	uint8_t g;
	uint8_t b;
	uint8_t w;

	static const CRGBW Red;
	static const CRGBW Green;
	static const CRGBW Blue;
	static const CRGBW White;
	static const CRGBW Black;
};

const CRGBW CRGBW::Red = {255, 0, 0, 0};
const CRGBW CRGBW::Green = {0, 255, 0, 0};
const CRGBW CRGBW::Blue = {0, 0, 255, 0};
const CRGBW CRGBW::White = {0, 0, 0, 255};
const CRGBW CRGBW::Black = {0, 0, 0, 0};

#define LED1 1
#define LED2 2
#define LED3 4
#define LEDALL (LED1 | LED2 | LED3)

void showRGBWLed(uint8_t led, const CRGBW &rgbw) {
	analogWrite(LED_PINS[led * 4 + 0], rgbw.r);
	analogWrite(LED_PINS[led * 4 + 1], rgbw.g);
	analogWrite(LED_PINS[led * 4 + 2], rgbw.b);
	analogWrite(LED_PINS[led * 4 + 3], rgbw.w);
}

void showRGBW(uint8_t led_mask, const CRGBW &rgbw) {
	if (led_mask & LED1) showRGBWLed(0, rgbw);
	if (led_mask & LED2) showRGBWLed(1, rgbw);
	if (led_mask & LED3) showRGBWLed(2, rgbw);
}

void colorBars() {
	showRGBW(LEDALL, CRGBW::Red);
	delay(500);
	showRGBW(LEDALL, CRGBW::Green);
	delay(500);
	showRGBW(LEDALL, CRGBW::Blue);
	delay(500);
	showRGBW(LEDALL, CRGBW::White);
	delay(500);
	showRGBW(LEDALL, CRGBW::Black);
}

void setup() {
	SerialUSB.begin(115200);

	// Status LED
	pinMode(LED_BUILTIN, OUTPUT);

	// Test colors
	colorBars();

	SerialUSB.println("{\"status\":\"ready\"}");
}

void loop() {
	while (SerialUSB.available()) {
		int c = SerialUSB.peek();
		if (c == ' ' || c == '\n' || c == '\r' || c == '\t')
			SerialUSB.read();
		else
			break;
	}

	if (SerialUSB.available()) {
		StaticJsonDocument<128> doc;

		auto error = deserializeJson(doc, SerialUSB);
		if (error) {
			// Error occurred
			StaticJsonDocument<64> result;
			result["error"] = error.c_str();
			serializeJson(result, SerialUSB);
			SerialUSB.println();
		} else {
			// No error
			showRGBW(
			    doc["led"].as<uint8_t>(),
			    {doc["r"].as<uint8_t>(), doc["g"].as<uint8_t>(),
			     doc["b"].as<uint8_t>(), doc["w"].as<uint8_t>()});

			StaticJsonDocument<64> result;
			result["success"] = true;
			serializeJson(result, SerialUSB);
			SerialUSB.println();
		}
	}
}
