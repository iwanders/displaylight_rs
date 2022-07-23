 
static mut USB_BUS: Option<UsbBusAllocator<UsbBusType>> = None;
static mut USB_SERIAL: Option<usbd_serial::SerialPort<UsbBusType>> = None;
static mut USB_DEVICE: Option<UsbDevice<UsbBusType>> = None;

// On the whole sharing;
// https://github.com/rust-embedded/wg/issues/294#issuecomment-454425980
// https://github.com/rust-embedded/wg/issues/294#issuecomment-456416517
// https://github.com/rust-embedded/wg/issues/294#issuecomment-456742114
// https://docs.rs/cortex-m/latest/cortex_m/interrupt/fn.free.html
//https://github.com/geomatsi/rust-blue-pill-tests/blob/master/src/bin/blink-timer-irq-safe.rs

// SerialPort implements a stream.
// CdcAcmClass implements packets...


use stm32f1xx_hal::pac::{self, interrupt, Interrupt, NVIC};
use stm32f1xx_hal::prelude::*;
use stm32f1xx_hal::usb::{Peripheral, UsbBus, UsbBusType};
use usb_device::{bus::UsbBusAllocator, prelude::*};
use usbd_serial::{SerialPort, USB_CLASS_CDC};

// use core::cell::RefCell;
use cortex_m::interrupt::Mutex;

use core::cell::RefCell;
use core::ops::DerefMut;

use crate::ringbuffer;

type RxBuffer = ringbuffer::RingBuffer<u8, { 64 + 1 }>;
type TxBuffer = ringbuffer::RingBuffer<u8, { 64 + 1 }>;

// Microcontroller to PC
static BUFFER_TX: Mutex<RefCell<Option<RxBuffer>>> = Mutex::new(RefCell::new(None));

// PC to microcontroller.
static BUFFER_RX: Mutex<RefCell<Option<TxBuffer>>> = Mutex::new(RefCell::new(None));

/// Serial object is the main interface to the ringbuffers that are serviced by the ISR.
pub struct Serial {}

impl Serial {
    pub fn new(usb: Peripheral) -> Self {
        // Unsafe to allow access to static variables
        unsafe {
            cortex_m::interrupt::free(|cs| {
                BUFFER_TX.borrow(cs).replace(Some(TxBuffer::new()));
                BUFFER_RX.borrow(cs).replace(Some(RxBuffer::new()));
            });

            let bus = UsbBus::new(usb);

            USB_BUS = Some(bus);

            USB_SERIAL = Some(SerialPort::new(USB_BUS.as_ref().unwrap()));

            let usb_dev =
                UsbDeviceBuilder::new(USB_BUS.as_ref().unwrap(), UsbVidPid(0x16c0, 0x27dd))
                    .manufacturer("Fake company")
                    .product("Serial port")
                    .serial_number("TEST")
                    .device_class(USB_CLASS_CDC)
                    .build();

            USB_DEVICE = Some(usb_dev);
        }

        unsafe {
            NVIC::unmask(Interrupt::USB_HP_CAN_TX);
            NVIC::unmask(Interrupt::USB_LP_CAN_RX0);
        }
        Serial {}
    }

    pub fn write(&mut self, data: &[u8]) {
        // let z = unsafe {cortex_m::interrupt::CriticalSection::new()};
        let mut data = data;

        // let mut buffer = unsafe { BUFFER_TX.as_mut().unwrap() };

        cortex_m::interrupt::free(|cs| {
            if let Some(ref mut buffer) = BUFFER_TX.borrow(cs).borrow_mut().deref_mut() {
                while !data.is_empty() {
                    let write_buffer = buffer.write_slice_mut();
                    let max_len = core::cmp::min(data.len(), write_buffer.len());
                    write_buffer[0..max_len].copy_from_slice(&data[0..max_len]);
                    buffer.write_advance(max_len);
                    data = &data[max_len..];
                }
            }
        });
    }

    pub fn service(&mut self) {
        usb_interrupt();
        write_from_buffer();
        read_to_buffer();
        usb_interrupt();
    }

    pub fn available(&self) -> usize {
        0
    }
}

#[interrupt]
fn USB_HP_CAN_TX() {
    usb_interrupt();
}

#[interrupt]
fn USB_LP_CAN_RX0() {
    usb_interrupt();
}

fn write_from_buffer() {
    cortex_m::interrupt::free(|cs| {
        let serial = unsafe { USB_SERIAL.as_mut().unwrap() };
        if let Some(ref mut buffer) = BUFFER_TX.borrow(cs).borrow_mut().deref_mut() {
            // let mut buffer = unsafe { BUFFER_TX.as_mut().unwrap() };
            let mut read_buffer = buffer.read_slice_mut();
            while !read_buffer.is_empty() {
                let count = read_buffer.len();
                serial.write(&read_buffer).ok();
                buffer.read_advance(count);
                read_buffer = buffer.read_slice_mut();
            }
        }
    });
}

fn read_to_buffer() {
    cortex_m::interrupt::free(|cs| {
        let serial = unsafe { USB_SERIAL.as_mut().unwrap() };
        if let Some(ref mut buffer) = BUFFER_RX.borrow(cs).borrow_mut().deref_mut() {
            // let mut buffer = unsafe { BUFFER_RX.as_mut().unwrap() };
            let write_buffer = buffer.write_slice_mut();
            // BUFFER_TX.borrow(&z).borrow_mut().replace(TxBuffer::new());
            // let mut buf = [0u8; 8];

            match serial.read(write_buffer) {
                Ok(count) if count > 0 => {
                    let adv = buffer.write_advance(count);
                }
                _ => {}
            }
        }
    });
} /**/

fn usb_interrupt() {
    cortex_m::interrupt::free(|v| {
        let usb_dev = unsafe { USB_DEVICE.as_mut().unwrap() };
        let serial = unsafe { USB_SERIAL.as_mut().unwrap() };

        if !usb_dev.poll(&mut [serial]) {
            return;
        }
    });
}
