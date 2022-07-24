# Displaylight firmware

## Test
```
cargo t --target x86_64-unknown-linux-gnu
```

## Flash and run
Run `openocd` in this directory, then;

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
