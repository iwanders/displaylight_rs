# Displaylight firmware

## Circuit
The pcb to connect the STM32f103 to the uses a [sacrificial led](https://hackaday.com/2017/01/20/cheating-at-5v-ws2812-control-to-use-a-3-3v-data-line/) to ensure the signal gets converted from 3.3v to 5.0v logic level.

<img width="100%" src="/firmware/doc/displaylight_pcb.svg">

## Test
```
cargo t --target x86_64-unknown-linux-gnu
```

## Flash and run
Run `openocd` in this directory, then in another terminal

```
cargo r --profile firmware
```


## Note on interrupts
```
usb_dev.poll(&mut [serial])
```

Is supposed to be called as many times as possible, preferably from an interrupt. Bit it seems that
enabling `NVIC::unmask(Interrupt::USB_LP_CAN_RX0);` and calling it from that interrupt causes the 
interrupt to fire indefinitely on itself.
