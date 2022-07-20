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
