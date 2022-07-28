//! Message structs adopted from the C++ implementation running on the microcontroller.

/*
//  Message definitions
struct RGB
{
  uint8_t R;
  uint8_t G;
  uint8_t B;
  uint32_t toUint32() const
  {
    return (R << 16) | (G << 8) | B;
  }
};
*/

#[repr(C, packed)]
#[derive(Default, Clone, Copy, Debug)]
/// Struct to represent the RGB state of a single led.
pub struct RGB {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

/*
enum MsgType : uint8_t
{
  NOP = 0,
  CONFIG = 1,
  COLOR = 2
};

*/
/// MsgType container to retrieve constants from. Is an enum on the C++ sid.
pub struct MsgType {}
impl MsgType {
    pub const NOP: u8 = 0;
    pub const CONFIG: u8 = 1;
    pub const COLOR: u8 = 2;
}
/*


struct Config
{
  //! If there has been activity, decay won't take place for decay_time_delay_ms milliseconds.
  uint32_t decay_time_delay_ms;  //!< 0 is disabled.

  //! The decay interval, after inactivity the decay will be performed every decay_interval_us microseconds.
  uint32_t decay_interval_us;

  //! The amount of decay that occurs each cycle.
  uint32_t decay_amount;

  float gamma_r;  //!< Gamma for the red channel.
  float gamma_g;  //!< Gamma for the green channel.
  float gamma_b;  //!< Gamma for the blue channel.
};
*/
#[repr(C, packed)]
#[derive(Clone, Copy, Debug)]
/// Config struct to change properties on the microcontroller.
pub struct Config {
    /// If there has been activity, decay won't take place for decay_time_delay_ms milliseconds.
    pub decay_time_delay_ms: u32,

    /// The decay interval, after inactivity the decay will be performed every decay_interval_us microseconds.
    pub decay_interval_us: u32,

    /// The amount of decay that occurs each cycle.
    pub decay_amount: u32,

    /// Gamma for the red channel.
    pub gamma_r: f32,
    /// Gamma for the green channel.
    pub gamma_g: f32,
    /// Gamma for the blue channel.
    pub gamma_b: f32,
}
impl Default for Config {
    /// Sets the defaults as they are in the firmware.
    fn default() -> Self {
        Config {
            decay_time_delay_ms: 1000,
            decay_interval_us: 1000,
            decay_amount: 1,
            gamma_r: 1.0,
            gamma_g: 1.3,
            gamma_b: 1.6,
        }
    }
}

/*

struct ColorData
{
  static constexpr const size_t leds_per_message{ 19 };
  static constexpr const size_t settings_show_after{ 1 << 0 };
  static constexpr const size_t settings_set_all{ 1 << 1 };
  uint16_t offset;
  uint8_t settings;
  RGB color[leds_per_message];  // takes 12 messages to send 228 bytes
};
*/
/// The number of led colors that can be sent in a single message.
const LEDS_PER_MESSAGE: usize = 19;
#[repr(C, packed)]
#[derive(Default, Clone, Copy, Debug)]
/// Struct to contain the color data for pixels as sent in a message.
pub struct ColorData {
    pub offset: u16,
    pub settings: u8,
    pub color: [RGB; LEDS_PER_MESSAGE],
}

impl ColorData {
    /// If this bit is set in settings, the led string will be 'drawn' after this message is processed.
    pub const SETTINGS_SHOW_AFTER: u8 = 1u8 << 0;

    /// If this bit is set, all leds in the led string will be set to the first color provided in
    /// the color array.
    pub const SETTINGS_SET_ALL: u8 = 1u8 << 1;

    /// The number of leds that are sent in a single message.
    pub const LEDS_PER_MESSAGE: usize = LEDS_PER_MESSAGE;
}

/*
struct Message
{
  MsgType type;
  uint8_t _[3];  // padding
  union {
    ColorData color;
    Config config;
    uint8_t raw[60];
  };
};  // exactly 64 bytes long = 1 usb packet.
*/
#[repr(C)]
#[derive(Clone, Copy)]
/// Union for the payload sent in the message.
pub union Payload {
    pub color: ColorData,
    pub config: Config,
    pub raw: [u8; 60],
}
impl Default for Payload {
    fn default() -> Self {
        Payload { raw: [0; 60] }
    }
}
use core::fmt;
use core::fmt::Debug;
impl Debug for Payload {
    /// Debug formatter for the payload always uses raw.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let as_raw = unsafe { &self.raw };
        f.debug_struct("Payload").field("raw", as_raw).finish()
    }
}

#[repr(C, packed)]
#[derive(Default, Clone, Copy)]
/// Message struct that will be sent across the wire to the microcontroller.
pub struct Message {
    pub msg_type: u8,
    pub _padding: [u8; 3],
    pub payload: Payload,
}
impl Debug for Message {
    /// Format a human readable version of this message. Interpreting the msg_type field.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.msg_type {
            MsgType::NOP => f
                .debug_struct("Message")
                .field("msg_type", &"nop")
                .finish(),
            MsgType::CONFIG => f
                .debug_struct("Message")
                .field("msg_type", &"config")
                .field("config", unsafe { &self.payload.config })
                .finish(),
            MsgType::COLOR => f
                .debug_struct("Message")
                .field("msg_type", &"color")
                .field("config", unsafe { &self.payload.color })
                .finish(),
            _ => f
                .debug_struct("Message")
                .field("msg_type", &"unknown")
                .finish(),
        }
    }
}

impl Message {
    pub fn as_bytes(&self) -> [u8; 64] {
        // Lets just do this here, alternatively we could pull in https://github.com/iwanders/huntsman/tree/master/struct_helper
        let mut res = [0u8; 64];
        unsafe {
            let rawptr = self as *const Self;
            let byte_ptr = rawptr as *const u8; // the reinterpret_cast
                                                // return a bounded slice of bytes for inspection.
            res[0..64].clone_from_slice(core::slice::from_raw_parts(
                byte_ptr,
                core::mem::size_of::<Self>(),
            ));
        }
        res
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config() {
        let mut m: Message = Default::default();
        m.msg_type = MsgType::CONFIG;
        m.payload.config.decay_time_delay_ms = 0xdeadbeef;
        m.payload.config.decay_interval_us = 0x01020304;
        m.payload.config.decay_amount = 0xF1F2F3F4;
        m.payload.config.gamma_r = 0.33333;
        m.payload.config.gamma_g = 1.0;
        m.payload.config.gamma_b = 0.6;
        let b = m.as_bytes();
        let expected = [
            1u8, 0, 0, 0, 239, 190, 173, 222, 4, 3, 2, 1, 244, 243, 242, 241, 59, 170, 170, 62, 0,
            0, 128, 63, 154, 153, 25, 63, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ];
        println!("{:?}", m);
        println!("{:?}", b);
        assert_eq!(b, expected);
    }

    #[test]
    fn test_color() {
        let mut msg: Message = Default::default();
        msg.msg_type = MsgType::COLOR;
        msg.payload.color.offset = 0x0102;
        msg.payload.color.settings = 0xAB;
        let mut colors: [RGB; ColorData::LEDS_PER_MESSAGE] = Default::default();
        for c in 0..ColorData::LEDS_PER_MESSAGE {
            colors[c].r = c as u8 * 3 + 0;
            colors[c].g = c as u8 * 3 + 1;
            colors[c].b = c as u8 * 3 + 2;
        }
        msg.payload.color.color = colors;

        let b = msg.as_bytes();
        let expected = [
            2u8, 0, 0, 0, 2, 1, 171, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17,
            18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39,
            40, 41, 42, 43, 44, 45, 46, 47, 48, 49, 50, 51, 52, 53, 54, 55, 56,
        ];
        println!("{:?}", msg);
        println!("{:?}", b);
        assert_eq!(b, expected);
    }
}
