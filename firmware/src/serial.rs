// On the whole sharing;
// https://github.com/rust-embedded/wg/issues/294#issuecomment-454425980
// https://github.com/rust-embedded/wg/issues/294#issuecomment-456416517
// https://github.com/rust-embedded/wg/issues/294#issuecomment-456742114
// https://docs.rs/cortex-m/latest/cortex_m/interrupt/fn.free.html
// https://github.com/geomatsi/rust-blue-pill-tests/blob/master/src/bin/blink-timer-irq-safe.rs
// https://github.com/adamgreig/ledeaf/blob/fbfed437c77f9bc4d83ea9fae4cee4e107af2e15/firmware/src/main.rs

// SerialPort implements a stream.
// CdcAcmClass implements packets...
use stm32f1xx_hal::pac::{interrupt, Interrupt, NVIC};
use stm32f1xx_hal::usb::{Peripheral, UsbBus, UsbBusType};
use usb_device::{bus::UsbBusAllocator, prelude::*};
use usbd_serial::{SerialPort, USB_CLASS_CDC};

use core::cell::RefCell;
use cortex_m::interrupt::Mutex;

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
    pub fn new() -> Self {
        Serial {}
    }
    pub fn init(usb: Peripheral) -> Self {
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

    pub fn write(&mut self, data: &[u8]) -> usize {
        let writer = unsafe { BUFFER_TO_HOST_WRITER.as_mut().unwrap() };
        let mut count = 0usize;
        for v in data.iter() {
            if let Ok(_) = writer.write_value(*v) {
                count += 1
            } else {
                break;
            }
        }
        return count;
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

use core::fmt::Error;
// Implement the Write trait for the serial port.
impl core::fmt::Write for Serial {
    fn write_str(&mut self, s: &str) -> Result<(), Error> {
        let mut b = s.as_bytes();
        while !b.is_empty() {
            let written = self.write(b);
            b = &b[written..];
            // Emergency service.
            if !b.is_empty() {
                self.service();
            }
        }
        Ok(())
    }
    // fn write_char(&mut self, c: char) -> Result { ... }
    // fn write_fmt(&mut self, args: Arguments<'_>) -> Result { ... }
}

/// Provide a println! macro similar to Rust does.
#[macro_export]
macro_rules! sprintln {
    () => ($crate::io::print("\n"));
    ($($arg:tt)*) => ({
        use core::fmt::Write;
        use core::fmt;
        let mut v = displaylight_fw::serial::Serial::new();
        core::fmt::write(&mut v, format_args!($($arg)*)).expect("Error occurred while trying to write in String");
        v.write_str("\n").expect("Shouldn't fail");
    })
}

#[interrupt]
fn USB_HP_CAN_TX() {
    usb_interrupt();
}

// #[interrupt]
// fn USB_LP_CAN_RX0() {
// usb_interrupt();
// }

fn write_from_buffer() {
    cortex_m::interrupt::free(|cs| {
        let mut serial_borrow = unsafe { USB_SERIAL.borrow(cs).borrow_mut() };
        let serial = serial_borrow.as_mut().unwrap();
        let z = unsafe { BUFFER_TO_HOST_READER.as_mut().unwrap() };
        while !z.is_empty() {
            let peeked = z.peek_value().unwrap();
            if let Ok(_v) = serial.write(&[*peeked]) {
                // ok, we could read it, now actually consume it.
                z.read_value().unwrap();
            } else {
                // we can't write anymore... yikes, lets service the serial port.
                // usb_interrupt();
                // No, we want to drop these bytes as they are mcu -> PC...
                // if we were to block here to service them, the mcu would stall if nothing on the
                // pc is consuming the bytes? Do we need some fancy logic?
                z.read_value().unwrap();
                break;
            }
        }
    });
}

fn read_to_buffer() {
    cortex_m::interrupt::free(|cs| {
        let mut serial_borrow = unsafe { USB_SERIAL.borrow(cs).borrow_mut() };
        let serial = serial_borrow.as_mut().unwrap();
        // let usb_dev = unsafe { USB_DEVICE.as_mut().unwrap() };
        let writer = unsafe { BUFFER_FROM_HOST_WRITER.as_mut().unwrap() };

        // Data coming from the PC... we really don't want to lose this, leave it in the usb
        // serial device from host writer is full.
        while !writer.is_full() {
            let mut buf = [0u8; 1];
            match serial.read(&mut buf) {
                Ok(count) if count > 0 => {
                    for i in 0..count {
                        let _res = writer.write_value(buf[i]);
                    }
                    if count == 0 {
                        break;
                    }
                }
                _ => {
                    break;
                }
            }
        }

        /*
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
        */
    });
}

fn usb_interrupt() {
    cortex_m::interrupt::free(|cs| {
        let mut serial_borrow = unsafe { USB_SERIAL.borrow(&cs).borrow_mut() };
        let serial = serial_borrow.as_mut().unwrap();
        let usb_dev = unsafe { USB_DEVICE.as_mut().unwrap() };

        if !usb_dev.poll(&mut [serial]) {
            return;
        }
    });
}
