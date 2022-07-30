// This started as the following examples;
// https://github.com/stm32-rs/stm32f1xx-hal/blob/f9b24f4d9bac7fc3c93764bd295125800944f53b/examples/blinky.rs
// https://github.com/stm32-rs/stm32f1xx-hal/blob/f9b24f4d9bac7fc3c93764bd295125800944f53b/examples/usb_serial.rs
// https://github.com/adamgreig/ledeaf/blob/fbfed437c77f9bc4d83ea9fae4cee4e107af2e15/firmware/src/main.rs
// https://github.com/thalesfragoso/keykey/blob/master/keykey/Cargo.toml
// https://github.com/rtic-rs/cortex-m-rtic

#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]

#[cfg(not(test))]
use panic_halt as _;

use cortex_m::asm::delay;
use cortex_m_rt::entry;
use stm32f1xx_hal::prelude::*;

use stm32f1xx_hal::pac::{self};
use stm32f1xx_hal::usb::Peripheral;

use displaylight_fw::lights;
use displaylight_fw::messages;
use displaylight_fw::serial;
use displaylight_fw::spi_ws2811;
use displaylight_fw::types::RGB;

use cortex_m::singleton;
// use displaylight_fw::sprintln;

#[cfg_attr(not(test), entry)]
fn main() -> ! {
    // Get access to the core peripherals from the cortex-m crate
    let _cp = cortex_m::Peripherals::take().unwrap();
    // Get access to the device specific peripherals from the peripheral access crate
    let dp = pac::Peripherals::take().unwrap();

    // Take ownership over the raw flash and rcc devices and convert them into the corresponding
    // HAL structs
    let mut flash = dp.FLASH.constrain();
    let rcc = dp.RCC.constrain();

    // Freeze the configuration of all the clocks in the system and store the frozen frequencies in
    // `clocks`
    // Set a real clock that allows usb.
    let clocks = rcc
        .cfgr
        .use_hse(8.MHz())
        .sysclk(48.MHz())
        .pclk1(24.MHz())
        .freeze(&mut flash.acr);

    assert!(clocks.usbclk_valid());

    // spi on bus B
    let mut gpiob = dp.GPIOB.split();
    let pins = (
        // (sck, miso, mosi)
        // gpiob.pb13.into_alternate_push_pull(&mut gpiob.crh),
        stm32f1xx_hal::spi::NoSck,
        // gpiob.pb14.into_floating_input(&mut gpiob.crh),
        stm32f1xx_hal::spi::NoMiso,
        gpiob.pb15.into_alternate_push_pull(&mut gpiob.crh),
    );

    // Set up the DMA device
    let dma = dp.DMA1.split();

    const REAL_LED_COUNT: usize = 228;
    const LED_OFFSET: usize = 1; // the sacrificial led
    const LEDS: usize = REAL_LED_COUNT + LED_OFFSET; // one sacrificial led.
    const BUFFER_SIZE: usize = spi_ws2811::Ws2811SpiDmaDriver::calculate_buffer_size(LEDS);

    // Create the led buffer, this is moved to the spi ws2811 driver.
    let buf = singleton!(: [u8; BUFFER_SIZE] = [0; BUFFER_SIZE]).unwrap();

    // Create the led color buffer, this allows updating the driver from this.
    let colors = singleton!(: [RGB; LEDS] = [RGB::BLACK; LEDS]).unwrap();

    // Finally, create the led management using the SPI bus and dma channel.
    let mut ws2811 =
        spi_ws2811::Ws2811SpiDmaDriver::new(dp.SPI2, pins, clocks, dma.5, &mut buf[..]);
    ws2811.prepare(colors);
    ws2811.update(); // Turn all leds off.

    // Create the lights manager
    let mut lights = lights::Lights::new(colors, LED_OFFSET);

    // counter_ms: Can wait from 2 ms to 65 sec for 16-bit timer
    // counter_us: Can wait from 2 μs to 65 ms for 16-bit timer
    // Start something to keep time.
    let mut my_timer = dp.TIM2.counter_us(&clocks);

    // We can't update the clock difference each cycle, as that rounds the millis to zero.
    let lights_time_update_interval = stm32f1xx_hal::time::ms(10);
    // Setup the timer with a timer period to wrap around.
    let timer_period = 64.millis();
    my_timer.start(timer_period).unwrap();

    // Keep track of the old time.
    let mut old = my_timer.now();

    // Setup usb serial
    let mut gpioa = dp.GPIOA.split();

    // BluePill board has a pull-up resistor on the D+ line.
    // Pull the D+ pin down to send a RESET condition to the USB bus.
    // This forced reset is needed only for development, without it host
    // will not reset your device when you upload new firmware.
    let mut usb_dp = gpioa.pa12.into_push_pull_output(&mut gpioa.crh);
    usb_dp.set_low();
    delay(clocks.sysclk().raw() / 100);

    let usb_dm = gpioa.pa11;
    let usb_dp = usb_dp.into_floating_input(&mut gpioa.crh);

    let usb = Peripheral {
        usb: dp.USB,
        pin_dm: usb_dm,
        pin_dp: usb_dp,
    };

    // With usb setup, the serial port handler can be setup;
    let mut s = serial::Serial::init(usb);

    // Setup the led;
    // Acquire the GPIOC peripheral
    let mut gpioc = dp.GPIOC.split();

    // Configure gpio C pin 13 as a push-pull output. The `crh` register is passed to the function
    // in order to configure the port. For pins 0-7, crl should be passed instead.
    let mut led = gpioc.pc13.into_push_pull_output(&mut gpioc.crh);

    // Counter to toggle the led.
    let mut led_toggle_counter: _ = stm32f1xx_hal::time::MicroSeconds::from_ticks(0);

    loop {
        // Service the serial port.
        s.service();

        // If there are bytes on the serial port to read, consume them all until we have a full
        // message.
        if s.available() {
            let mut msg_buff = [0u8; messages::Message::LENGTH];
            let mut read_buff = &mut msg_buff[..];
            while !read_buff.is_empty() {
                let read = s.read_into(read_buff);
                read_buff = &mut read_buff[read..];
                s.service(); // Also service the port in this loop.
            }
            lights.incoming(&msg_buff);
        }

        // This uses the timer to update the clock in the lights object every timer interval
        // it gracefully handles wrap around of the timer as it resets.
        let current = my_timer.now();
        let diff = stm32f1xx_hal::time::MicroSeconds::from_ticks(
            (((current.ticks() as i64 - old.ticks() as i64) + timer_period.ticks() as i64)
                % timer_period.ticks() as i64) as u32,
        );

        // Finally, if the lights time update interval has passed, update the time for the lights.
        // This is only done periodically to avoid the difference rounding to zero.
        if diff > lights_time_update_interval {
            old = current;
            lights.clock_update(diff.to_micros() as u64);
            led_toggle_counter = led_toggle_counter + diff;
        }

        // Always, perform the update for the light manager.
        lights.perform_update(&mut ws2811);

        // Toggle the builtin led to indicate we lock up or panic.
        if led_toggle_counter > stm32f1xx_hal::time::ms(50) {
            led_toggle_counter = stm32f1xx_hal::time::MicroSeconds::from_ticks(0);
            led.toggle();
        }
    }
}
