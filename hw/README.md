# colorui.rs hardware support

This folder contains the source for a *Black pill* board (STM32F103C8), which has 12 PWM outputs (supports 3 RGBW LED strips with the appropriate drivers, for example ULN2803A).

## Usage

Flash using a ST-Link v2, with platformio.org installed:

```bash
pio run -t upload
```

## Author

Vincent Tavernier <vince.tavernier@gmail.com>
