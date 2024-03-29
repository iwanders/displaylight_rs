//! A module to control LED lights attached to a microcontroller.
mod messages;
use messages::{ColorData, Message, MsgType};

use serialport::SerialPort;

pub use messages::{Config, RGB};

/// Object to control led lights.
pub struct Lights {
    port: Box<dyn SerialPort>,
    limit_factor: f32,
}

use std::error::Error;
use std::time::Duration;

impl Lights {
    /// Create a new Lights instance, attaching to the provided serial port.
    pub fn new(port_name: &str) -> Result<Lights, Box<dyn Error>> {
        let port = serialport::new(port_name, 9600) // Baud rate is a dummy anyway.
            .timeout(Duration::from_millis(100))
            .open()
            .map_err(|ref e| format!("Port '{}' not available: {}", &port_name, e))?;
        Ok(Lights {
            port,
            limit_factor: 1.0,
        })
    }

    /// Set the brightness limiting factor (0.0 - 1.0), this reduces the overall brightness for any
    /// values set through [`Lights::fill`] or [`Lights::set_leds`]. This can be helpful if the
    /// power supply is inadequate.
    pub fn set_limit_factor(&mut self, factor: f32) {
        self.limit_factor = factor;
    }

    /// Set the configuration on the microcontroller to the provided struct.
    pub fn set_config(&mut self, config: &Config) -> Result<(), Box<dyn Error>> {
        let mut msg: Message = Message {
            msg_type: MsgType::CONFIG,
            ..Default::default()
        };
        msg.payload.config = *config;

        self.port.write_all(&msg.as_bytes())?;
        Ok(())
    }

    /// Fill the entire string of leds with the provided color.
    pub fn fill(&mut self, r: u8, g: u8, b: u8) -> Result<(), Box<dyn Error>> {
        let mut msg: Message = Message {
            msg_type: MsgType::COLOR,
            ..Default::default()
        };
        msg.payload.color.offset = 0;
        msg.payload.color.settings = ColorData::SETTINGS_SET_ALL | ColorData::SETTINGS_SHOW_AFTER;
        let mut colors: [RGB; ColorData::LEDS_PER_MESSAGE] = Default::default();
        colors[0].r = (r as f32 * self.limit_factor) as u8;
        colors[0].g = (g as f32 * self.limit_factor) as u8;
        colors[0].b = (b as f32 * self.limit_factor) as u8;
        msg.payload.color.color = colors;

        self.port.write_all(&msg.as_bytes())?;
        Ok(())
    }

    /// Set the leds to the provided pixel values.
    pub fn set_leds(&mut self, pixels: &[RGB]) -> Result<(), Box<dyn Error>> {
        // chunk the pixels into LEDS_PER_MESSAGE.
        let chunk_count =
            (pixels.len() as f32 / ColorData::LEDS_PER_MESSAGE as f32).ceil() as usize;
        for (i, chunk) in pixels.chunks(ColorData::LEDS_PER_MESSAGE).enumerate() {
            let is_final = i + 1 == chunk_count;
            let mut msg: Message = Message {
                msg_type: MsgType::COLOR,
                ..Default::default()
            };
            msg.payload.color.offset = (i * ColorData::LEDS_PER_MESSAGE) as u16;
            // Only if it is the last chunk, write the data.
            msg.payload.color.settings = if is_final {
                ColorData::SETTINGS_SHOW_AFTER
            } else {
                0
            };
            let mut colors: [RGB; ColorData::LEDS_PER_MESSAGE] = Default::default();
            for c in 0..std::cmp::min(pixels.len(), ColorData::LEDS_PER_MESSAGE) {
                colors[c].r = (chunk[c].r as f32 * self.limit_factor) as u8;
                colors[c].g = (chunk[c].g as f32 * self.limit_factor) as u8;
                colors[c].b = (chunk[c].b as f32 * self.limit_factor) as u8;
            }
            msg.payload.color.color = colors;
            self.port.write_all(&msg.as_bytes())?;
        }
        Ok(())
    }
}

/// Helper function to list the available ports.
pub fn available_ports() -> Result<Vec<serialport::SerialPortInfo>, serialport::Error> {
    serialport::available_ports()
}
