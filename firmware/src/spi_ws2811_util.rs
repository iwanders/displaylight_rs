
use crate::types::{RGB};

// From datasheet:
//            _______
// 0 code:   |T0H    |_T0L____|
//            _________
// 1 code:   |T1H      |_T1L__|
//
// ret:      |____Tret________|
//
// 3 MHz gives us 3.3333333333333335e-07 per bit, which is 0.3 us
// OctoWs2811 has;
// T0H: 0.3 us
// T1H: 0.75 us
// TH_TL: 1.25 us
// 1.25 / 0.3 = ~ 4.1 may be able to put two bits in a single SPI byte?
// Start with
// 0 bit represented by spi byte: 0b10000000
// 1 bit represented by spi byte: 0b11100000

// Hardcode on 3Mhz spi bus for now.
const WS2811_0BIT: u8 = 0b10000000;
const WS2811_1BIT: u8 = 0b11100000;



pub fn convert_color_to_buffer(colors: &[RGB], buffer: &mut [u8])  {
    assert_eq!(colors.len() * 3 * 8, buffer.len());
    let mut buffer_i = 0usize;
    for c in colors.iter() {
        for b in &[c.g, c.r, c.b] {
            for i in (0..8).rev() {
                let bit_set = if ((b >> i) & 1) == 1 { true } else { false};
                buffer[buffer_i] = if bit_set {WS2811_1BIT} else {WS2811_0BIT};
                buffer_i = buffer_i + 1;
            }
        }
    }
}

