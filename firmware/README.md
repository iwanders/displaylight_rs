# displaylight_fw

Firmware for displaylight_rs, target platform is an STM32F103 'blue pill' development board. Uses
288 rgb leds of the ws2811 type.

## In bulletpoints

- The main logic is in the Lights structure. The main file holds the setup and main loop.
- If communication stops (PC turns off) leds fade gracefully.
- TIM2 is used for global timekeeping, providing time difference.
- USB Serial port is available and facilitates easy debug printing from anywhere using SPSC ringbuffers.
- The SPI bus is used to create the signal for the LEDs.
- Each ws2811 bit is sent by using a full byte on the SPI bus.
- SPI bus runs at 6 MHz and transfers 228 * 3 * 8 = 5472 bytes for each full led update.
- SPI write operation is done through a DMA channel, freeing up the MCU while writing to the leds.
- Gamma correction is applied just before the LED's RGB channels are expanded into SPI bytes.

## Circuit
The pcb to connect the STM32F103 to the uses a [sacrificial led](https://hackaday.com/2017/01/20/cheating-at-5v-ws2812-control-to-use-a-3-3v-data-line/) to ensure the signal gets converted from 3.3v to 5.0v logic level.

<img width="100%" src="/firmware/doc/displaylight_pcb.svg">

## Test
```
cargo t --target x86_64-unknown-linux-gnu
```

## Flash and run
Run `openocd` in this directory, then in another terminal run;

```
cargo r --profile firmware
```

## Note on interrupts
```
usb_dev.poll(&mut [serial])
```

Is supposed to be called as many times as possible, preferably from an interrupt. But it seems that
enabling `NVIC::unmask(Interrupt::USB_LP_CAN_RX0);` and calling it from that interrupt causes the 
interrupt to fire indefinitely on itself. Instead, this is just called from the main program loop.
