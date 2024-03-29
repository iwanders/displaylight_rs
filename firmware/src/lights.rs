use crate::messages::{ColorData, Config, Message, ReceivedMessage};
use crate::spi_ws2811::Ws2811SpiDmaDriver;
use crate::sprintln;
use crate::types::RGB;

pub fn set_rgbw(leds: &mut [RGB], offset: usize) {
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

pub fn set_color(leds: &mut [RGB], color: &RGB) {
    for v in leds.iter_mut() {
        *v = *color;
    }
}

pub fn set_limit(leds: &mut [RGB], value: u8) {
    for v in leds.iter_mut() {
        v.limit(value);
    }
}

/// Object to manage the led lights, keeps the internal state, manages the gamma filters and decay.
#[derive(Default)]
pub struct Lights {
    /// Configuration for the decay and gamma tables.
    config: Config,

    /// Current time in usec.
    current_time: u64,
    /// Timestamp at which the last event was received
    last_msg: u64,
    /// Timestamp of the last decay operation.
    last_decay: u64,

    /// Gamma correction element.
    gamma: crate::gamma::Gamma,

    /// Internal led state to use.
    leds: &'static mut [RGB],

    /// Offset to start counting, to drop the sacrificial led if present.
    led_offset: usize,

    /// Flag to indicate led values should be set.
    needs_update: bool,
}

impl Lights {
    /// Test method that allows inspecting the leds.
    #[cfg(test)]
    pub fn get_leds(&self) -> &[RGB] {
        self.leds
    }

    /// Create a new lights object, taking a RGB buffer and offset from where the real leds start.
    /// This offset is zero if all leds are to be used, one if there's a sacrificial led to do the
    /// voltage conversion.
    pub fn new(leds: &'static mut [RGB], led_offset: usize) -> Self {
        let config = Config::default();
        let gamma = crate::gamma::Gamma::generate(config.gamma_r, config.gamma_g, config.gamma_b);
        Lights {
            leds,
            led_offset,
            gamma,
            config,
            ..Default::default()
        }
    }

    /// Interprets Message::LENGTH bytes from the slice as a message.
    pub fn incoming(&mut self, data_bytes: &[u8]) {
        assert_eq!(data_bytes.len(), Message::LENGTH);
        let msg = Message::from_bytes(data_bytes);
        if msg.is_none() {
            sprintln!("Got bad payload: {:?}", data_bytes); // booo.
            return;
        }
        let msg = msg.unwrap();

        // Update the last msg received.
        self.last_msg = self.current_time;

        match msg {
            ReceivedMessage::Nop => {}
            ReceivedMessage::ColorData(color_data) => {
                let mut color_data = color_data;

                // set all with the first pixel?
                if (color_data.settings & ColorData::SETTINGS_SET_ALL) != 0 {
                    self.leds[self.led_offset..].fill(color_data.color[0]);
                } else {
                    // Add the sacrificial led offset.
                    color_data.offset = color_data.offset + self.led_offset as u16;

                    // Copy to the appropriate section of the led string.
                    let start = core::cmp::min(color_data.offset as usize, self.leds.len());
                    let end = core::cmp::min(
                        (color_data.offset + ColorData::LEDS_PER_MESSAGE as u16) as usize,
                        self.leds.len(),
                    );
                    let to_update = &mut self.leds[start..end];
                    to_update.copy_from_slice(&color_data.color[..to_update.len()])
                }

                // If we should set the leds, set the update flag.
                if (color_data.settings & ColorData::SETTINGS_SHOW_AFTER) != 0 {
                    self.needs_update = true;
                }
            }
            ReceivedMessage::Config(config) => {
                // Only update the gamma tables if they changed.
                if (self.config.gamma_r != config.gamma_r)
                    || (self.config.gamma_g != config.gamma_g)
                    || (self.config.gamma_b != config.gamma_b)
                {
                    // Calculating the gamma tables is expensive, but should only need to happen once.
                    self.gamma = crate::gamma::Gamma::generate(
                        self.config.gamma_r,
                        self.config.gamma_g,
                        self.config.gamma_b,
                    );
                    self.needs_update = true;
                }
                self.config = config;
            }
        }
    }

    fn update_decay(&mut self) {
        // Determine if we should decay;
        if (self.current_time - self.last_msg) > (self.config.decay_time_delay_ms * 1000) as u64 {
            if (self.current_time - self.last_decay) > (self.config.decay_interval_us as u64) {
                self.last_decay = self.current_time;
                self.needs_update = true;
                // perform decay.
                let sub_value = self.config.decay_amount;
                for v in self.leds.iter_mut() {
                    v.r = v.r.saturating_sub(sub_value as u8);
                    v.g = v.g.saturating_sub(sub_value as u8);
                    v.b = v.b.saturating_sub(sub_value as u8);
                }
            }
        }
    }

    pub fn clock_update(&mut self, dt: u64) {
        self.current_time = dt + self.current_time;
    }

    pub fn perform_update(&mut self, ws2811: &mut Ws2811SpiDmaDriver) {
        self.update_decay();
        if ws2811.is_ready() && self.needs_update {
            ws2811.prepare_filter(&self.leds, &|rgb| {
                let mut v = [rgb; 1];
                self.gamma.apply(&mut v);
                v[0]
            });
            self.needs_update = false;
            ws2811.update();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn state_checks() {
        const ssa: u8 = ColorData::SETTINGS_SHOW_AFTER;
        let led_state_msg = [
            2u8, 0, 0, 0, 15, 0, ssa, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17,
            18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39,
            40, 41, 42, 43, 44, 45, 46, 47, 48, 49, 50, 51, 52, 53, 54, 55, 56,
        ];
        static mut leds: [RGB; 228] = [RGB::BLACK; 228];
        let mut lights = Lights::new(unsafe { &mut leds }, 0);
        lights.incoming(&led_state_msg);
        // println!("Leds: {:?}", lights.get_leds());
        assert_eq!(lights.get_leds()[0], RGB::BLACK);
    }
}
