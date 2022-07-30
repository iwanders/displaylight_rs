use crate::types::RGB;

// SPI-bus based ws2811 driver, just like the other cool kids do.

// https://github.com/stm32-rs/stm32f1xx-hal/blob/f9b24f4d9bac7fc3c93764bd295125800944f53b/examples/spi-dma.rs
// https://github.com/stm32-rs/stm32f1xx-hal/blob/f9b24f4d9bac7fc3c93764bd295125800944f53b/examples/adc-dma-circ.rs
// Ideally we make a ringbuffer and keep writing... but lets leave that for now.

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
// Changed to 6Mhz because... faster is better.
const WS2811_0BIT: u8 = 0b11000000;
const WS2811_1BIT: u8 = 0b11111100;

/// Function to expand the color bytes into spi bytes.
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

use stm32f1xx_hal::dma::Transfer;
use stm32f1xx_hal::dma::TxDma;
use stm32f1xx_hal::gpio::Alternate;
use stm32f1xx_hal::gpio::PushPull;
use stm32f1xx_hal::gpio::CRH;
use stm32f1xx_hal::spi::NoMiso;
use stm32f1xx_hal::spi::NoSck;
use stm32f1xx_hal::spi::Spi2NoRemap;
use stm32f1xx_hal::spi::{Mode, Phase, Polarity, Spi};

// This is all pretty bad and hardcoded, but at least all the logic around the ws2811 driving is
// in one place. Complete with flip-flopping between having the data available for updating and
// performing the SPI transaction.

use stm32f1xx_hal::prelude::*;
type DmaType = stm32f1xx_hal::dma::dma1::C5;
type SpiPort = stm32f1xx_hal::pac::SPI2;
type SpiPins = (
    NoSck,
    NoMiso,
    stm32f1xx_hal::gpio::Pin<Alternate<PushPull>, CRH, 'B', 15_u8>,
);
type SpiDma = TxDma<Spi<SpiPort, Spi2NoRemap, SpiPins, u8>, DmaType>;
type TransferType = Transfer<stm32f1xx_hal::dma::R, &'static mut [u8], SpiDma>;
type StaticBuffer = &'static mut [u8];

/// Struct to hold the components if no transfer is ongoing.
struct Pending {
    spi_dma: SpiDma,
    buffer: StaticBuffer,
}
impl Pending {
    /// Split the pending struct into the spi dma bus and the data buffer.
    pub fn split(self) -> (SpiDma, StaticBuffer) {
        (self.spi_dma, self.buffer)
    }
}

/// Spi DMA driver struct, either pending is populate OR transfer is populated.
/// Flow after setup is:
///   ws2811.prepare(&colors); // updates the internal buffer
///   ws2811.update(); // Starts the transfer
///   while !ws2811.is_ready() {} // block until raed to prepare again.
pub struct Ws2811SpiDmaDriver {
    pending: Option<Pending>,
    transfer: Option<TransferType>,
}

impl Ws2811SpiDmaDriver {
    /// Just to ensure we have some zeros at the start.
    const PREAMBLE_COUNT: usize = 1;

    /// LOW period to set the pixels; 25 usec... at 6 Mhz, that is 150 clocks, that's 18.75 bytes.
    /// Lets say 21, so 7 pixels.
    const POST_COUNT: usize = 7;

    /// RGB is 3 channels, each bit is a full byte, so 8.
    const BYTES_PER_LED: usize = 3 * 8;

    /// Calculate the size of the static buffer necessary to hold the data for a number of leds.
    pub const fn calculate_buffer_size(led_count: usize) -> usize {
        (led_count + Self::PREAMBLE_COUNT + Self::POST_COUNT) * Self::BYTES_PER_LED
    }

    /// Setup the spi bus using the clock, pins, dma bus and static buffer.
    pub fn new(
        spi: stm32f1xx_hal::pac::SPI2,
        pins: SpiPins,
        clocks: stm32f1xx_hal::rcc::Clocks,
        dma: DmaType,
        buffer: &'static mut [u8],
    ) -> Self {
        let spi_mode = Mode {
            polarity: Polarity::IdleLow,
            phase: Phase::CaptureOnFirstTransition,
        };
        let spi = Spi::spi2(spi, pins, spi_mode, 6.MHz(), clocks);
        let spi_dma = spi.with_tx_dma(dma);

        let pending = Some(Pending { spi_dma, buffer });
        Ws2811SpiDmaDriver {
            pending,
            transfer: None,
        }
    }

    /// Trigger the transfer start and update all the leds to the internal state.
    pub fn update(&mut self) {
        self.finalize_transfer();
        if let Some(pending) = self.pending.take() {
            let (spi_dma, buffer) = pending.split();
            let transfer = spi_dma.write(buffer);
            self.transfer = Some(transfer);
        }
    }

    /// Returns whether the transfer is complete, only makes sense when a transfer is ongoing.
    fn is_transfer_complete(&self) -> bool {
        if let Some(transfer) = &self.transfer {
            return transfer.is_done();
        }
        return false;
    }

    /// Function to finalize a transfer if it can be finalized, this moves the transfer back
    /// into the buffer and spi_dma pending object.
    fn finalize_transfer(&mut self) {
        if !self.is_transfer_complete() {
            return;
        }
        if let Some(transfer) = self.transfer.take() {
            let (buffer, spi_dma) = transfer.wait();
            let pending = Some(Pending { spi_dma, buffer });
            self.pending = pending;
        }
    }

    /// Returns whether prepare can be called.
    pub fn is_ready(&mut self) -> bool {
        self.finalize_transfer();
        self.pending.is_some()
    }

    /// Prepare the internal buffer with the provided led size.
    pub fn prepare(&mut self, colors: &[RGB]) {
        if let Some(pending) = &mut self.pending {
            convert_color_to_buffer(
                colors,
                &mut pending.buffer[(Self::BYTES_PER_LED * Self::PREAMBLE_COUNT)
                    ..((colors.len() + Self::PREAMBLE_COUNT) * Self::BYTES_PER_LED)],
            );
        }
    }

    /// Prepare with a filter to be applied on each RGB value.
    pub fn prepare_filter(&mut self, colors: &[RGB], filter: &dyn Fn(RGB) -> RGB) {
        if let Some(pending) = &mut self.pending {
            for (i, rgb) in colors.iter().enumerate() {
                let filtered_rgb = filter(*rgb);
                convert_color_to_buffer(
                    &[filtered_rgb],
                    &mut pending.buffer[(Self::BYTES_PER_LED * (Self::PREAMBLE_COUNT + i))
                        ..((Self::PREAMBLE_COUNT + i + 1) * Self::BYTES_PER_LED)],
                );
            }
        }
    }
}

// Using a denser (6Mhz but with 2 bits per byte) doesn't work unfortunately;
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
