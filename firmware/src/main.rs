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
use stm32f1xx_hal::prelude::*; //, timer::Timer

// use embedded_hal::digital::v2::OutputPin;
// use embedded_hal::digital::v2::PinState::{High, Low};

use stm32f1xx_hal::pac::{self}; // , interrupt, Interrupt, NVIC
                                // use stm32f1xx_hal::prelude::*;
use stm32f1xx_hal::usb::Peripheral;

use displaylight_fw::gamma;
use displaylight_fw::serial;
use displaylight_fw::spi_ws2811;
use displaylight_fw::types::RGB;

use displaylight_fw::sprintln;

use cortex_m::singleton;

static mut G_V: usize = 0;


fn set_rgbw(leds: &mut [RGB], offset: usize) {
    for i in 0..leds.len() {
        let v = (i + offset) % 4;
        if v == 0 {
            leds[i] = RGB::RED;
        } else if v == 1 {
            leds[i] = RGB::GREEN;
        } else if v == 2 {
            leds[i] = RGB::BLUE;
        } else if v == 3 {
            leds[i] = RGB::WHITE;
        }
    }
}

fn set_color(leds: &mut [RGB], color: &RGB) {
    for v in leds.iter_mut() {
        *v = *color;
    }
}

fn set_limit(leds: &mut [RGB], value: u8) {
    for v in leds.iter_mut() {
        v.limit(value);
    }
}

/**/
// global timekeeping.
use core::sync::atomic::AtomicUsize;
use core::sync::atomic::Ordering;
 use stm32f1xx_hal::pac::TIM2;

use core::cell::RefCell;
use cortex_m::interrupt::Mutex;
const SERVICE_INTERVAL_MS: u32 = 10;
static GLOBAL_CLOCK_US: AtomicUsize = AtomicUsize::new(0);
use stm32f1xx_hal::timer::CounterUs;
static GLOBAL_TIMER: Mutex<RefCell<Option<CounterUs<TIM2>>>> = Mutex::new(RefCell::new(None));

// TIM2 interrupt, service usb every 5ms and keeps track of global timekeeping
use stm32f1xx_hal::pac::{interrupt, Interrupt, NVIC};
use core::borrow::BorrowMut;

#[interrupt]
fn TIM2() {
    GLOBAL_CLOCK_US.fetch_add((SERVICE_INTERVAL_MS * 1000) as usize, Ordering::Release);
    let c = GLOBAL_CLOCK_US.load(Ordering::Relaxed);
    cortex_m::interrupt::free(|cs| {
        GLOBAL_TIMER.borrow(cs).borrow_mut().as_mut().unwrap().clear_interrupt(stm32f1xx_hal::timer::Event::Update);
    });
}

fn clock_ms() -> stm32f1xx_hal::time::MilliSeconds {
    let v = unsafe { 
        cortex_m::interrupt::free(|cs| {
            GLOBAL_TIMER.borrow(cs).borrow_mut().as_mut().unwrap().now().ticks()
        })
    };
    let v = v / 1000;
    let c = GLOBAL_CLOCK_US.load(Ordering::Relaxed);
    let c = c + v as usize;
    stm32f1xx_hal::time::MilliSeconds::from_ticks((c / 1000) as u32)
}

fn clock_us() -> stm32f1xx_hal::time::MicroSeconds {
    let v = unsafe { 
        cortex_m::interrupt::free(|cs| {
            GLOBAL_TIMER.borrow(cs).borrow_mut().as_mut().unwrap().now().ticks()
        })
    };
    let c = GLOBAL_CLOCK_US.load(Ordering::Relaxed);
    let c = c + v as usize;
    stm32f1xx_hal::time::MicroSeconds::from_ticks(c as u32)
}

fn clock_us_u64() -> u64 {
    let v = unsafe { 
        cortex_m::interrupt::free(|cs| {
            GLOBAL_TIMER.borrow(cs).borrow_mut().as_mut().unwrap().now().ticks()
        })
    };
    let c = GLOBAL_CLOCK_US.load(Ordering::Relaxed) as u64;
    c + v as u64
}



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

    const LEDS: usize = 226;
    const BUFFER_SIZE: usize = spi_ws2811::Ws2811SpiDmaDriver::calculate_buffer_size(LEDS);

    // Create the led buffer, this is moved to the spi ws2811 driver.
    let buf = singleton!(: [u8; BUFFER_SIZE] = [0; BUFFER_SIZE]).unwrap();

    // Create the led color buffer, this allows updating the driver from this.
    let mut colors: [RGB; LEDS] = [RGB::BLACK; LEDS];

    let mut ws2811 =
        spi_ws2811::Ws2811SpiDmaDriver::new(dp.SPI2, pins, clocks, dma.5, &mut buf[..]);
    ws2811.prepare(&colors);
    ws2811.update();

    // Create the gamma correction tables.
    let filter = gamma::Gamma::correction();
    // let filter = gamma::Gamma::linear();



    // delay_clock.delay_ms(2u16);

    // counter_ms: Can wait from 2 ms to 65 sec for 16-bit timer
    // counter_us: Can wait from 2 μs to 65 ms for 16-bit timer
    let mut my_timer = dp.TIM2.counter_us(&clocks);
    my_timer.start(SERVICE_INTERVAL_MS.millis()).unwrap();

    my_timer.listen(stm32f1xx_hal::timer::Event::Update);
    // Start the usb service routine.
    cortex_m::interrupt::free(|cs| *GLOBAL_TIMER.borrow(cs).borrow_mut() = Some(my_timer));

    unsafe {
        NVIC::unmask(Interrupt::TIM2);
    }

    let mut old = clock_ms();



    let mut delay_clock = dp.TIM3.delay_us(&clocks);
    delay_clock.delay_ms(100u16);


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
    s.service();

    // delay_clock.delay_ms(10000u16);

    let mut v = 0usize;
    let mut led_state: bool = false;
    let mut c = 0usize;

    let mut start_us = clock_us_u64();
    loop {
        v += 1;
        unsafe {
            G_V = v;
            core::ptr::read_volatile(&G_V);
        }
        s.service();
        // continue;

        let current = clock_ms();
        let diff = stm32f1xx_hal::time::MilliSeconds::from_ticks(
            current.ticks().wrapping_sub(old.ticks()),
        );

        // if transfer.is_done() {
        // delay.delay_ms(2u16); // need some delay here to make the 150 us low.
        // sprintln!("done {}, going into wait", my_timer.now());
        // s.service();

        // let (buf, spi_dma) = transfer.wait();
        // sprintln!("starting {} w", my_timer.now());
        // s.service();

        // transfer = spi_dma.write(buf);

        // sprintln!("exiting write {}", my_timer.now());
        // s.service();
        // }
        // It's taking 16ms :< -> 8ms now, that should be sufficient... 125Hz update rate.

        // sprintln!("current: {}, clock: {}, old: {} diff: {}", current, clock_ms(), old, diff);
        if diff > stm32f1xx_hal::time::ms(20) {
            // my_timer.reset()
            // dp.TIM2.reset();
            old = current;
        } else {
            continue;
        }

        if ws2811.is_ready() {
            set_rgbw(&mut colors, 2);
            let cu8 = (c % 255) as u8;
            // set_color(&mut colors, &RGB{r: cu8, g: 0, b: 0});
            // set_color(&mut colors, &RGB{r: 0, g: cu8, b: 0});
            // set_color(&mut colors, &RGB{r: 0, g: 0, b: cu8});
            // let v = current.ticks();
            c += 1;
            sprintln!("{}  {} \n", c, c % 255);
            set_limit(&mut colors, cu8);
            // set_limit(&mut colors, 1);
            filter.apply(&mut colors);
            ws2811.prepare(&colors);
            ws2811.update();
        }
        if led_state {
            led.set_low();
        } else {
            led.set_high();
        }
        led_state = !led_state;

        // let tic = my_timer.now();
        // delay_clock.delay_ms(10u16);
        // let toc = my_timer.now();

        // sprintln!("{} {}, {}\n", v);

        while s.available() {
            if let Some(v) = s.read() {
                s.write(&[v - 0x20]);
            } else {
                break;
            }
        }
    }
}
