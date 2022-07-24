// On the whole sharing;
// https://github.com/rust-embedded/wg/issues/294#issuecomment-454425980
// https://github.com/rust-embedded/wg/issues/294#issuecomment-456416517
// https://github.com/rust-embedded/wg/issues/294#issuecomment-456742114
// https://docs.rs/cortex-m/latest/cortex_m/interrupt/fn.free.html
// https://github.com/geomatsi/rust-blue-pill-tests/blob/master/src/bin/blink-timer-irq-safe.rs
// https://github.com/adamgreig/ledeaf/blob/fbfed437c77f9bc4d83ea9fae4cee4e107af2e15/firmware/src/main.rs

// SerialPort implements a stream.
// CdcAcmClass implements packets...

mod own_interrupt {
    use core::arch::asm;
    use core::sync::atomic::{compiler_fence, Ordering};

    #[inline]
    pub unsafe fn enable() {
        compiler_fence(Ordering::SeqCst);
        asm!("cpsie i", options(nomem, nostack)); //, preserves_flags
    }
    #[inline]
    pub unsafe fn disable() {
        asm!("cpsid i", options(nomem, nostack)); //, preserves_flags
        compiler_fence(Ordering::SeqCst);
    }
}

use stm32f1xx_hal::pac::{self, interrupt, Interrupt, NVIC};
use stm32f1xx_hal::prelude::*;
use stm32f1xx_hal::usb::{Peripheral, UsbBus, UsbBusType};
use usb_device::{bus::UsbBusAllocator, prelude::*};
use usbd_serial::{SerialPort, USB_CLASS_CDC};

// use core::cell::RefCell;
use cortex_m::interrupt::Mutex;

use core::cell::RefCell;
use core::ops::DerefMut;

// use crate::ringbuffer;
use crate::spsc;

const RX_BUFFER_SIZE: usize = 64;
type RxBuffer = spsc::SpScRingbuffer<u8, { RX_BUFFER_SIZE }>;
type RxWriter<'a> = spsc::Writer<'a, u8, RX_BUFFER_SIZE>;
type RxReader<'a> = spsc::Reader<'a, u8, RX_BUFFER_SIZE>;

const TX_BUFFER_SIZE: usize = 64;
type TxBuffer = spsc::SpScRingbuffer<u8, { TX_BUFFER_SIZE }>;
type TxWriter<'a> = spsc::Writer<'a, u8, TX_BUFFER_SIZE>;
type TxReader<'a> = spsc::Reader<'a, u8, TX_BUFFER_SIZE>;

// Microcontroller to PC
static mut BUFFER_TO_HOST: TxBuffer = TxBuffer::new();
static mut BUFFER_TO_HOST_WRITER: Option<TxWriter> = None;
static mut BUFFER_TO_HOST_READER: Option<TxReader> = None;

// PC to microcontroller.
static mut BUFFER_FROM_HOST: RxBuffer = RxBuffer::new();
static mut BUFFER_FROM_HOST_WRITER: Option<RxWriter> = None;
static mut BUFFER_FROM_HOST_READER: Option<RxReader> = None;

// Actual usb things.
static mut USB_BUS: Option<UsbBusAllocator<UsbBusType>> = None;
static mut USB_SERIAL: Mutex<RefCell<Option<usbd_serial::SerialPort<UsbBusType>>>> =
    Mutex::new(RefCell::new(None));
static mut USB_DEVICE: Option<UsbDevice<UsbBusType>> = None;

/// Serial object is the main interface to the ringbuffers that are serviced by the ISR.
pub struct Serial {}

impl Serial {
    pub fn new(usb: Peripheral) -> Self {
        // Unsafe to allow access to static variables
        unsafe {
            let bus = UsbBus::new(usb);
            USB_BUS = Some(bus);

            USB_SERIAL = Mutex::new(RefCell::new(Some(SerialPort::new(
                USB_BUS.as_ref().unwrap(),
            ))));

            let usb_dev =
                UsbDeviceBuilder::new(USB_BUS.as_ref().unwrap(), UsbVidPid(0x16c0, 0x27dd))
                    .manufacturer("Fake company")
                    .product("Serial port")
                    .serial_number("TEST")
                    .device_class(USB_CLASS_CDC)
                    .build();

            USB_DEVICE = Some(usb_dev);

            {
                let (reader, writer) = BUFFER_TO_HOST.split();
                BUFFER_TO_HOST_WRITER = Some(writer);
                BUFFER_TO_HOST_READER = Some(reader);
            }
            {
                let (reader, writer) = BUFFER_FROM_HOST.split();
                BUFFER_FROM_HOST_WRITER = Some(writer);
                BUFFER_FROM_HOST_READER = Some(reader);
            }
        }

        unsafe {
            NVIC::unmask(Interrupt::USB_HP_CAN_TX);
            // Enabling this and servicing the usb device causes cascading interrupts.
            // NVIC::unmask(Interrupt::USB_LP_CAN_RX0);
        }
        Serial {}
    }

    pub fn write(&mut self, data: &[u8]) {
        let writer = unsafe { BUFFER_TO_HOST_WRITER.as_mut().unwrap() };
        for v in data {
            let r = writer.write_value(*v);
            if r.is_err() {
                break;
            }
        }
    }

    pub fn service(&mut self) {
        usb_interrupt();
        write_from_buffer();
        read_to_buffer();
    }

    pub fn available(&self) -> bool {
        let reader = unsafe { BUFFER_FROM_HOST_READER.as_mut().unwrap() };
        !reader.is_empty()
    }

    pub fn read(&self) -> Option<u8> {
        let reader = unsafe { BUFFER_FROM_HOST_READER.as_mut().unwrap() };
        reader.read_value()
    }

    pub fn read_into(&self, buffer: &mut [u8]) -> usize {
        let mut i = 0usize;
        while self.available() && i < buffer.len() {
            if let Some(v) = self.read() {
                buffer[i] = v;
                i += 1;
            } else {
                break;
            }
        }
        i
    }
}

#[interrupt]
fn USB_HP_CAN_TX() {
    usb_interrupt();
}

// #[interrupt]
// fn USB_LP_CAN_RX0() {
// usb_interrupt();
// }

fn go_to_overflow() -> ! {
    loop {}
}

fn write_from_buffer() {
    cortex_m::interrupt::free(|cs| {
        let cs = unsafe { &cortex_m::interrupt::CriticalSection::new() };
        let mut serial_borrow = unsafe { USB_SERIAL.borrow(cs).borrow_mut() };
        let serial = serial_borrow.as_mut().unwrap();
        let usb_dev = unsafe { USB_DEVICE.as_mut().unwrap() };
        let z = unsafe { BUFFER_TO_HOST_READER.as_mut().unwrap() };
        while !z.is_empty() {
            if let Ok(v) = serial.write(&[z.read_value().unwrap()]) {
            } else {
                // go_to_overflow();
            }
        }
    });
}

fn read_to_buffer() {
    cortex_m::interrupt::free(|cs| {
        let cs = unsafe { &cortex_m::interrupt::CriticalSection::new() };
        let mut serial_borrow = unsafe { USB_SERIAL.borrow(cs).borrow_mut() };
        let serial = serial_borrow.as_mut().unwrap();
        let usb_dev = unsafe { USB_DEVICE.as_mut().unwrap() };
        let writer = unsafe { BUFFER_FROM_HOST_WRITER.as_mut().unwrap() };

        let mut buf = [0u8; 64];

        match serial.read(&mut buf) {
            Ok(count) if count > 0 => {
                for i in 0..count {
                    let res = writer.write_value(buf[i]);
                    if res.is_err() {
                        break;
                    }
                }
            }
            _ => {}
        }
    });
}

fn usb_interrupt() {
    cortex_m::interrupt::free(|cs| {
        let cs = unsafe { &cortex_m::interrupt::CriticalSection::new() };
        let mut serial_borrow = unsafe { USB_SERIAL.borrow(&cs).borrow_mut() };
        let serial = serial_borrow.as_mut().unwrap();
        let usb_dev = unsafe { USB_DEVICE.as_mut().unwrap() };

        if !usb_dev.poll(&mut [serial]) {
            return;
        }
    });
}
