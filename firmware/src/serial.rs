static mut USB_BUS: Option<UsbBusAllocator<UsbBusType>> = None;
static mut USB_SERIAL: Option<usbd_serial::SerialPort<UsbBusType>> = None;
static mut USB_DEVICE: Option<UsbDevice<UsbBusType>> = None;

// On the whole sharing;
// https://github.com/rust-embedded/wg/issues/294#issuecomment-454425980
// https://github.com/rust-embedded/wg/issues/294#issuecomment-456416517
// https://github.com/rust-embedded/wg/issues/294#issuecomment-456742114
// https://docs.rs/cortex-m/latest/cortex_m/interrupt/fn.free.html

// SerialPort implements a stream.
// CdcAcmClass implements packets...

use stm32f1xx_hal::pac::{self, interrupt, Interrupt, NVIC};
use stm32f1xx_hal::prelude::*;
use stm32f1xx_hal::usb::{Peripheral, UsbBus, UsbBusType};
use usb_device::{bus::UsbBusAllocator, prelude::*};
use usbd_serial::{SerialPort, USB_CLASS_CDC};

use core::cell::RefCell;
use cortex_m::interrupt::Mutex;


use crate::ringbuffer;

type RxBuffer = ringbuffer::RingBuffer::<u8, {64 + 1 } >;
type TxBuffer = ringbuffer::RingBuffer::<u8, {64 + 1 } >;

// Microcontroller to PC
static buffer_tx: Mutex<RefCell<Option<RxBuffer>>> = Mutex::new(RefCell::new(None));

// PC to microcontroller.
static buffer_rx: Mutex<RefCell<Option<TxBuffer>>> = Mutex::new(RefCell::new(None));


/// Serial object is the main interface to the ringbuffers that are serviced by the ISR.
pub struct Serial {}

impl Serial {
    pub fn new(usb: Peripheral) -> Self {
        // Unsafe to allow access to static variables
        unsafe {
            {
                let z = cortex_m::interrupt::CriticalSection::new();
                buffer_rx.borrow(&z).borrow_mut().replace(RxBuffer::new());
                buffer_tx.borrow(&z).borrow_mut().replace(TxBuffer::new());
            }

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
        let z = unsafe {cortex_m::interrupt::CriticalSection::new()};

        let mut buffer =  buffer_tx.borrow(&z).borrow_mut();
        let write_buffer = (*buffer).as_mut().unwrap().write_slice_mut();
        //left.copy_from_slice(subst);
        // write_buffer.copy_from_slice(data);
        write_buffer[0..data.len()].copy_from_slice(data);
        (*buffer).as_mut().unwrap().write_advance(data.len());
    }

    pub fn service(&mut self) {
        let serial = unsafe { USB_SERIAL.as_mut().unwrap() };
        let z = unsafe {cortex_m::interrupt::CriticalSection::new()};
        write_to_buffer(serial, &z);
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

fn write_to_buffer(serial: &mut usbd_serial::SerialPort<UsbBusType>, z: &cortex_m::interrupt::CriticalSection)
{
    let mut buffer =  buffer_tx.borrow(&z).borrow_mut();
    let read_buffer = (*buffer).as_mut().unwrap().read_slice_mut();
    let count = read_buffer.len();
    serial.write(&read_buffer).ok();
    (*buffer).as_mut().unwrap().read_advance(count);
}

fn usb_interrupt() {
    let usb_dev = unsafe { USB_DEVICE.as_mut().unwrap() };
    let serial = unsafe { USB_SERIAL.as_mut().unwrap() };

    if !usb_dev.poll(&mut [serial]) {
        return;
    }

    let z = unsafe {cortex_m::interrupt::CriticalSection::new()};
    // Reading
    {
        let mut buffer =  buffer_rx.borrow(&z).borrow_mut();
        let write_buffer = (*buffer).as_mut().unwrap().write_slice_mut();
        // buffer_tx.borrow(&z).borrow_mut().replace(TxBuffer::new());
        // let mut buf = [0u8; 8];

        match serial.read(write_buffer) {
            Ok(count) if count > 0 => {
                let adv = (*buffer).as_mut().unwrap().write_advance(count);
                // serial.write(&buf[0..count]).ok();
            }
            _ => {}
        }
    }

    // Writing
    {
        let mut buffer =  buffer_tx.borrow(&z).borrow_mut();
        let read_buffer = (*buffer).as_mut().unwrap().read_slice_mut();
        let count = read_buffer.len();
        serial.write(&read_buffer).ok();
        (*buffer).as_mut().unwrap().read_advance(count);
    }

    write_to_buffer(serial, &z);
}
