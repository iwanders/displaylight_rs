# displaylight_rs

![display_light_active](https://raw.githubusercontent.com/iwanders/DisplayLight/master/displaylight.gif)

I've continued development on the `screen_capture` crate [here](https://github.com/iwanders/screen_capture), this workspace now
consumes that crate.

This [Rust][rust] workspace is a rewrite of my [DisplayLight](https://github.com/iwanders/DisplayLight)
project. It colors leds mounted behind the monitor with the colors shown on the display at that location, this is known as [bias lighting](https://en.wikipedia.org/wiki/Bias_lighting), (example [gif](https://github.com/iwanders/DisplayLight/blob/master/displaylight.gif)).

Approach is still the same as in the original project:
- Screen capture takes a snapshot of the screen and keeps it in shared memory.
  - Uses X11's shared memory extension [Xshm](https://en.wikipedia.org/wiki/MIT-SHM) on Linux.
  - Uses the [Desktop Duplication API](https://docs.microsoft.com/en-us/windows/win32/direct3ddxgi/desktop-dup-api) on Windows (with help of [windows-rs][windows-rs]).
- Black border detection is performed to find the interesting region on the screen. Only allowing smooth transitions between border sizes to prevent flickerring in dark scenes.
- Zones are created from this region of interest, each zone will map to one led.
- Zones are sampled, sampled colors averaged to determine zone value and thus led color.
- Led string is updated with the obtained values.

The hardware is now based on an STM32F103 'blue pill' development board. It is further described in the [firmware](firmware) directory, the firmware is also written in Rust.

Running on either Windows and on Linux is a matter of `cargo run --release`. Configuration lives in [config](displaylight/config/) and is selected based on the operating system. Performance is identical to the C++ version, running at ~3% of an i7-4770TE core when sampling a 1920x1080 image at 60 Hz.

The [screen_capture](screen_capture) crate used to obtain the screen captures is completely stand alone and ~~could~~ has been used outside of this project.


## displaylight_fw

Firmware for displaylight_rs, target platform is an STM32F103 'blue pill' development board. Uses
288 rgb leds of the ws2811 type.

### In bulletpoints

- The main logic is in the Lights structure. The main file holds the setup and main loop.
- If communication stops (PC turns off) leds fade gracefully.
- TIM2 is used for global timekeeping, providing time difference.
- USB Serial port is available and facilitates easy debug printing from anywhere using SPSC ringbuffers.
- The SPI bus is used to create the signal for the LEDs.
- Each ws2811 bit is sent by using a full byte on the SPI bus.
- SPI bus runs at 6 MHz and transfers 228 * 3 * 8 = 5472 bytes for each full led update.
- SPI write operation is done through a DMA channel, freeing up the MCU while writing to the leds.
- Gamma correction is applied just before the LED's RGB channels are expanded into SPI bytes.

### Circuit
The pcb to connect the STM32F103 to the uses a [sacrificial led](https://hackaday.com/2017/01/20/cheating-at-5v-ws2812-control-to-use-a-3-3v-data-line/) to ensure the signal gets converted from 3.3v to 5.0v logic level.

<img width="100%" src="/firmware/doc/displaylight_pcb.svg">

### Test
```
cargo t --target x86_64-unknown-linux-gnu
```

### Flash and run
Run `openocd` in this directory, then in another terminal run;

```
cargo r --profile firmware
```

### Note on interrupts
```
usb_dev.poll(&mut [serial])
```

Is supposed to be called as many times as possible, preferably from an interrupt. But it seems that
enabling `NVIC::unmask(Interrupt::USB_LP_CAN_RX0);` and calling it from that interrupt causes the
interrupt to fire indefinitely on itself. Instead, this is just called from the main program loop.

### License
License is `MIT OR Apache-2.0`.

[rust]: https://www.rust-lang.org/
[windows-rs]: https://github.com/microsoft/windows-rs
