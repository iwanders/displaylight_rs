
use crate::types::{RGB};

/*
From datasheet:
           _______
0 code:   |T0H    |_T0L____|
           _________
1 code:   |T1H      |_T1L__|
//
ret:      |____Tret________|
//
3 MHz gives us 3.3333333333333335e-07 per bit, which is 0.3 us
OctoWs2811 has;
T0H: 0.3 us
T1H: 0.75 us
TH_TL: 1.25 us
1.25 / 0.3 = ~ 4.1 may be able to put two bits in a single SPI byte?
Start with
0 bit represented by spi byte: 0b10000000
1 bit represented by spi byte: 0b11100000

Datasheet however states;
T0H: 0.5 us  +/- 150ns
T0L: 2.0 us  +/- 150ns
T1L: 1.3 us  +/- 150ns
T1H: 1.2 us  +/- 150ns
RES: > 50us

0.5 / 2.5 = 0.2 ~= 0.25
(1.3 / 2.5) ~= 0.5

Maybe, at a lower clock frequency, we can actually cram two bits into each byte? That's a 50% saving
on the expanded data...

2MHz gives 1 bit at 0.5us
// Nope, didn't work, everything goes white.

*/

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


// Nope... :(
pub mod dense {
use super::*;
const WS2811_0BIT: u8 = 0b1000;
const WS2811_1BIT: u8 = 0b1100;

pub fn convert_color_to_buffer(colors: &[RGB], buffer: &mut [u8])  {
    assert_eq!(colors.len() * 3 * 8 / 2, buffer.len());
    let mut buffer_i = 0usize;
    for c in colors.iter() {
        for b in &[c.g, c.r, c.b] {
            for i in (0..8).rev() {
                let bit_set = if ((b >> i) & 1) == 1 { true } else { false};
                if i % 2 == 0 {
                    buffer[buffer_i] |= (if bit_set {WS2811_1BIT} else {WS2811_0BIT}) << 4;
                    buffer_i = buffer_i + 1;
                } else {
                    buffer[buffer_i] = if bit_set {WS2811_1BIT} else {WS2811_0BIT};
                }
            }
        }
    }
}
}


#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn state_checks() {
        let mut colors = [RGB::RED, RGB::GREEN, RGB::BLUE, RGB::WHITE];
        let mut low_buff : [u8; 4 * 3 * 8] = [0; 4 * 3 * 8];
        convert_color_to_buffer(&colors, &mut low_buff);
        println!("{:x?}", low_buff);
        let mut dense_buff : [u8; 4 * 3 * 8 / 2] = [0; 4 * 3 * 8 / 2];
        dense::convert_color_to_buffer(&colors, &mut dense_buff);
        println!("{:x?}", dense_buff);
    }
}
