# displaylight_rs

This [Rust][rust] workspace is a rewrite of my [DisplayLight](https://github.com/iwanders/DisplayLight)
project. It colors leds mounted behind the monitor with the colors shown on the display at that location, this is known as [bias lighting](https://en.wikipedia.org/wiki/Bias_lighting), sometimes refered to as `Ambilight`.

Steps are still the same as in the original project:
- Screen capture takes a snapshot of the screen and keeps it in shared memory.
  - Uses X11's shared memory extension [Xshm](https://en.wikipedia.org/wiki/MIT-SHM) on Linux.
  - Uses the [Desktop Duplication API](https://docs.microsoft.com/en-us/windows/win32/direct3ddxgi/desktop-dup-api) on Windows (with help of [windows-rs][windows-rs]).
- Black border detection is performed to find the interesting region on the screen.
- Zones are created from this region of interest, each zone will map to one led.
- Zones are sampled, sampled colors averaged to determine zone value.
- Led string is updated with the obtained values.

## Usage
Usage requires some hardware, most specifically a microcontroller running the [firmware](https://github.com/iwanders/DisplayLight/tree/master/firmware). If using something else, the [lights](lights) crate will need some changing to ensure communication with the hardware is correct.

Running on either Windows and on Linux is a matter of `cargo run --release`. Configuration lives in [config](displaylight/config/) and is selected based on the operating system. Performance is identical to the C++ version, running at ~3% of an i7-4770TE core when sampling a 1920x1080 image at 60 Hz.

The [screen_capture](screen_capture) crate used to obtain the screen captures is completely stand alone and could be used outside of this project.

## License
License is `MIT OR Apache-2.0`.

[rust]: https://www.rust-lang.org/
[windows-rs]: https://github.com/microsoft/windows-rs
