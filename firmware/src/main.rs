//! This example is a mix of:
//! https://github.com/stm32-rs/stm32f1xx-hal/blob/f9b24f4d9bac7fc3c93764bd295125800944f53b/examples/blinky.rs
//! and
//! https://github.com/stm32-rs/stm32f1xx-hal/blob/f9b24f4d9bac7fc3c93764bd295125800944f53b/examples/usb_serial.rs
//!
//! This assumes that a LED is connected to pc13 as is the case on the blue pill board.
//!
//! Note: Without additional hardware, PC13 should not be used to drive an LED, see page 5.1.2 of
//! the reference manual for an explanation. This is not an issue on the blue pill.

// https://github.com/adamgreig/ledeaf/blob/fbfed437c77f9bc4d83ea9fae4cee4e107af2e15/firmware/src/main.rs
// https://github.com/thalesfragoso/keykey/blob/master/keykey/Cargo.toml
// https://github.com/rtic-rs/cortex-m-rtic

// #![deny(unsafe_code)]

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

// use displaylight_fw::sprintln;
use cortex_m::singleton;

#[cfg_attr(not(test), entry)]
fn main() -> ! {
    // Get access to the core peripherals from the cortex-m crate
    let _cp = cortex_m::Peripherals::take().unwrap();
    // Get access to the device specific peripherals from the peripheral access crate
    let dp = pac::Peripherals::take().unwrap();

    // Configure the syst timer to trigger an update every second
    // let mut timer = Timer::syst(cp.SYST, &clocks).counter_hz();
    // timer.start(5.Hz()).unwrap();

    // Take ownership over the raw flash and rcc devices and convert them into the corresponding
    // HAL structs
    let mut flash = dp.FLASH.constrain();
    let rcc = dp.RCC.constrain();

    // Freeze the configuration of all the clocks in the system and store the frozen frequencies in
    // `clocks`
    // let clocks = rcc.cfgr.freeze(&mut flash.acr);
    // Set a real clock that allows usb.
    let clocks = rcc
        .cfgr
        .use_hse(8.MHz())
        .sysclk(48.MHz())
        .pclk1(24.MHz())
        .freeze(&mut flash.acr);

    assert!(clocks.usbclk_valid());

    // Acquire the GPIOC peripheral
    let mut gpioc = dp.GPIOC.split();

    // Configure gpio C pin 13 as a push-pull output. The `crh` register is passed to the function
    // in order to configure the port. For pins 0-7, crl should be passed instead.
    let mut led = gpioc.pc13.into_push_pull_output(&mut gpioc.crh);

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

    let mut ws2811 =
        spi_ws2811::Ws2811SpiDmaDriver::new(dp.SPI2, pins, clocks, dma.5, &mut buf[..]);
    ws2811.prepare(colors);
    ws2811.update();

    // Create the lights struct.
    let mut lights = lights::Lights::new(colors, LED_OFFSET);

    // counter_ms: Can wait from 2 ms to 65 sec for 16-bit timer
    // counter_us: Can wait from 2 μs to 65 ms for 16-bit timer
    // Start something to keep time.
    let mut my_timer = dp.TIM2.counter_us(&clocks);
    my_timer.start(64.millis()).unwrap();

    let mut old = my_timer.now();

    // Create something that can perform delays.
    // let mut delay_clock = dp.TIM3.delay_us(&clocks);
    // delay_clock.delay_ms(100u16);

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

    let mut s = serial::Serial::init(usb);


    loop {
        s.service();

        if s.available() {
            let mut msg_buff = [0u8; messages::Message::LENGTH];
            let mut read_buff = &mut msg_buff[..];
            while !read_buff.is_empty() {
                let read = s.read_into(read_buff);
                read_buff = &mut read_buff[read..];
                s.service();
            }
            lights.incoming(&msg_buff);
        }

        let current = my_timer.now();
        let diff = stm32f1xx_hal::time::MicroSeconds::from_ticks(
            current.ticks().wrapping_sub(old.ticks()),
        );

        lights.perform_update(diff.ticks() as u64, &mut ws2811);

        if diff > stm32f1xx_hal::time::ms(50) {
            old = current;
        } else {
            continue;
        }

        led.toggle();
    }
}
