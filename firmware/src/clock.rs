
// global timekeeping.
use core::sync::atomic::AtomicUsize;
use core::sync::atomic::Ordering;
use stm32f1xx_hal::pac::TIM2;

type TIMER = stm32f1xx_hal::pac::TIM2;

use stm32f1xx_hal::prelude::*; //, timer::Timer
use core::cell::RefCell;
use cortex_m::interrupt::Mutex;
const CLOCK_OVERFLOW_INTERVAL_US: u32 = 32768;
static GLOBAL_CLOCK_US: AtomicUsize = AtomicUsize::new(0);
use stm32f1xx_hal::timer::CounterUs;
static GLOBAL_TIMER: Mutex<RefCell<Option<CounterUs<TIMER>>>> = Mutex::new(RefCell::new(None));

// TIM2 interrupt, service usb every 5ms and keeps track of global timekeeping
use stm32f1xx_hal::pac::{interrupt, Interrupt, NVIC};
use core::borrow::BorrowMut;

#[interrupt]
fn TIM2() {
    GLOBAL_CLOCK_US.fetch_add((CLOCK_OVERFLOW_INTERVAL_US) as usize, Ordering::Release);
    let c = GLOBAL_CLOCK_US.load(Ordering::Relaxed);
    cortex_m::interrupt::free(|cs| {
        GLOBAL_TIMER.borrow(cs).borrow_mut().as_mut().unwrap().clear_interrupt(stm32f1xx_hal::timer::Event::Update);
    });
}

pub fn clock_ms() -> stm32f1xx_hal::time::MilliSeconds {
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

pub fn clock_us_u64() -> u64 {
    let v = unsafe { 
        cortex_m::interrupt::free(|cs| {
            GLOBAL_TIMER.borrow(cs).borrow_mut().as_mut().unwrap().now().ticks()
        })
    };
    let c = GLOBAL_CLOCK_US.load(Ordering::Relaxed) as u64;
    c + v as u64
}

pub fn clock_us() -> stm32f1xx_hal::time::MicroSeconds {
    stm32f1xx_hal::time::MicroSeconds::from_ticks(clock_us_u64() as u32)
}

pub fn setup_clock(timer: TIMER, clocks: stm32f1xx_hal::rcc::Clocks)
{
    let mut my_timer = timer.counter_us(&clocks);
    my_timer.start(CLOCK_OVERFLOW_INTERVAL_US.micros()).unwrap();

    my_timer.listen(stm32f1xx_hal::timer::Event::Update);
    // Start the usb service routine.
    cortex_m::interrupt::free(|cs| *GLOBAL_TIMER.borrow(cs).borrow_mut() = Some(my_timer));

    unsafe {
        NVIC::unmask(Interrupt::TIM2);
    }
}
