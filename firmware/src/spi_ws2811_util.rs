use crate::types::RGB;

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

use stm32f1xx_hal::pac::{self}; // , interrupt, Interrupt, NVIC
use stm32f1xx_hal::spi::{Mode, Phase, Polarity, Spi};

// Hardcode on 3Mhz spi bus for now.
// Changed to 6Mhz because... faster is better.
const WS2811_0BIT: u8 = 0b11000000;
const WS2811_1BIT: u8 = 0b11111100;

pub fn convert_color_to_buffer(colors: &[RGB], buffer: &mut [u8]) {
    assert_eq!(colors.len() * 3 * 8, buffer.len());
    let mut buffer_i = 0usize;
    for c in colors.iter() {
        for b in &[c.g, c.r, c.b] {
            for i in (0..8).rev() {
                let bit_set = if ((b >> i) & 1) == 1 { true } else { false };
                buffer[buffer_i] = if bit_set { WS2811_1BIT } else { WS2811_0BIT };
                buffer_i = buffer_i + 1;
            }
        }
    }
}

// impl<SPI, REMAP, PINS, FrameSize> SpiReadWrite<FrameSize> for Spi<SPI, REMAP, PINS, FrameSize>
// where
    // SPI: Instance,
use stm32f1xx_hal::spi::Spi2NoRemap;
use stm32f1xx_hal::spi::NoSck;use stm32f1xx_hal::spi::NoMiso;
use stm32f1xx_hal::gpio::Alternate;
use stm32f1xx_hal::gpio::PushPull;
use stm32f1xx_hal::gpio::CRH;
use stm32f1xx_hal::dma::TxDma;
use stm32f1xx_hal::dma::Transfer;


use stm32f1xx_hal::prelude::*; //, timer::Timer
type DmaType = stm32f1xx_hal::dma::dma1::C5;
type SpiPort = stm32f1xx_hal::pac::SPI2;
type SpiPins = (NoSck, NoMiso, stm32f1xx_hal::gpio::Pin<Alternate<PushPull>, CRH, 'B', 15_u8>);
// type SpiType = Spi<stm32f1xx_hal::pac::SPI2, Spi2NoRemap, SpiPins, u8>;
type SpiDma = TxDma<Spi<SpiPort, Spi2NoRemap, SpiPins, u8>, DmaType>;
type TransferType = Transfer<stm32f1xx_hal::dma::R, &'static mut [u8], SpiDma>;

// impl<REMAP, PINS> Spi<pac::SPI2, REMAP, PINS, u8> 
type StaticBuffer = &'static mut [u8];
struct Pending {
    spi_dma: SpiDma,
    buffer: StaticBuffer,
}
impl Pending {
    pub fn split(self) -> (SpiDma, StaticBuffer) {
        (self.spi_dma, self.buffer)
    }
}

pub struct Ws2811SpiDmaDriver {
    // spi: SpiType,
    // dma: DmaType,
    pending: Option<Pending>,
    transfer: Option<TransferType>,
    // transfer: Option<T>,
}

impl Ws2811SpiDmaDriver {
    pub fn new(spi: stm32f1xx_hal::pac::SPI2, pins: SpiPins, clocks: stm32f1xx_hal::rcc::Clocks, dma: DmaType, buffer: &'static mut [u8]) -> Self {

        let spi_mode = Mode {
            polarity: Polarity::IdleLow,
            phase: Phase::CaptureOnFirstTransition,
        };
        let spi = Spi::spi2(spi, pins, spi_mode, 6.MHz(), clocks);
        let spi_dma = spi.with_tx_dma(dma);
        // spi_dma + 3;
        
        // let dma = bus.with_tx_dma(dma.5);
        // stm32f1xx_hal::dma::Transfer
        let pending = Some(Pending{spi_dma, buffer});
        Ws2811SpiDmaDriver{pending, transfer: None}
    }

    pub fn start_update(&mut self) {
        if let Some(pending) = self.pending.take() {
            let (spi_dma, buffer) = pending.split();
            let transfer = spi_dma.write(buffer);
            self.transfer = Some(transfer);
        }
    }
}

// impl<SPI, REMAP, PINS, FrameSize> SpiReadWrite<FrameSize> for Spi<SPI, REMAP, PINS, FrameSize>
// where
    // SPI: Instance,
    // FrameSize: Copy,
// {

// Nope... :(
pub mod dense {
    use super::*;
    const WS2811_0BIT: u8 = 0b1000;
    const WS2811_1BIT: u8 = 0b1100;

    pub fn convert_color_to_buffer(colors: &[RGB], buffer: &mut [u8]) {
        assert_eq!(colors.len() * 3 * 8 / 2, buffer.len());
        let mut buffer_i = 0usize;
        for c in colors.iter() {
            for b in &[c.g, c.r, c.b] {
                for i in (0..8).rev() {
                    let bit_set = if ((b >> i) & 1) == 1 { true } else { false };
                    if i % 2 == 0 {
                        buffer[buffer_i] |= (if bit_set { WS2811_1BIT } else { WS2811_0BIT }) << 4;
                        buffer_i = buffer_i + 1;
                    } else {
                        buffer[buffer_i] = if bit_set { WS2811_1BIT } else { WS2811_0BIT };
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
        let mut low_buff: [u8; 4 * 3 * 8] = [0; 4 * 3 * 8];
        convert_color_to_buffer(&colors, &mut low_buff);
        println!("{:x?}", low_buff);
        let mut dense_buff: [u8; 4 * 3 * 8 / 2] = [0; 4 * 3 * 8 / 2];
        dense::convert_color_to_buffer(&colors, &mut dense_buff);
        println!("{:x?}", dense_buff);
    }
}
