//! Defines traits used by the desktop_frame crate.
use crate::raster_image::RasterImage;

#[derive(Debug, Default, Copy, Clone, Eq, PartialEq)]
#[repr(C)]
#[repr(align(4))]
/// Struct to represent a single pixel.
pub struct RGB {
    pub b: u8,
    pub g: u8,
    pub r: u8,
}

impl RGB {
    pub fn from_i32(v: i32) -> Self {
        // Checked godbolt, this evaporates to a single 'mov' and 'and' instruction.
        RGB {
            r: ((v >> 16) & 0xFF) as u8,
            g: ((v >> 8) & 0xFF) as u8,
            b: (v & 0xFF) as u8,
        }
    }

    pub fn black() -> RGB {
        RGB { r: 0, g: 0, b: 0 }
    }
    pub fn yellow() -> RGB {
        RGB {
            r: 255,
            g: 255,
            b: 0,
        }
    }
    pub fn cyan() -> RGB {
        RGB {
            r: 0,
            g: 255,
            b: 255,
        }
    }
    pub fn magenta() -> RGB {
        RGB {
            r: 255,
            g: 0,
            b: 255,
        }
    }
    pub fn white() -> RGB {
        RGB {
            r: 255,
            g: 255,
            b: 255,
        }
    }
}

#[derive(Debug, Default, Copy, Clone, Eq, PartialEq)]
/// Struct to represent the resolution.
pub struct Resolution {
    pub width: u32,
    pub height: u32,
}

/// Trait for something that represents an image.
pub trait Image {
    /// Returns the width of the image.
    fn get_width(&self) -> u32;

    /// Returns the height of the image.
    fn get_height(&self) -> u32;

    /// Returns a specific pixel's value. The x must be less then width, y less than height.
    fn get_pixel(&self, x: u32, y: u32) -> RGB;

    /// Returns the raw data buffer behind this image.
    fn get_data(&self) -> Option<&[RGB]> {
        None
    }

    /// Dump a ppm file to disk.
    fn write_ppm(&self, filename: &str) -> std::io::Result<()> {
        use std::fs::File;
        use std::io::prelude::*;
        let mut file = File::create(filename)?;
        file.write_all(b"P3\n")?;
        let width = self.get_width();
        let height = self.get_height();
        file.write_all(format!("{} {}\n", width, height).as_ref())?;
        file.write_all(b"255\n")?;
        for y in 0..height {
            let mut v: String = Default::default();
            v.reserve(4 * 3 * width as usize);
            for x in 0..width {
                let color = self.get_pixel(x, y);
                use std::fmt::Write;
                write!(v, "{} {} {} ", color.r, color.g, color.b).unwrap();
            }
            file.write_all(v.as_ref())?;
            file.write_all(b"\n")?;
        }
        Ok(())
    }

    /// Dump a bmp file to disk, mostly because windows can't open ppm.
    fn write_bmp(&self, filename: &str) -> std::io::Result<()> {
        // Adopted from https://stackoverflow.com/a/62946358
        use std::fs::File;
        use std::io::prelude::*;
        let mut file = File::create(filename)?;
        let width = self.get_width();
        let height = self.get_height();
        let pad = (((width as i32) * -3) & 3) as u32;
        let total = 54 + 3 * width * height + pad * height;
        let head: [u32; 7] = [total, 0, 54, 40, width, height, (24 << 16) | 1];
        let head_left = [0u32; 13 - 7];

        file.write_all(b"BM")?;
        file.write_all(
            &head
                .iter()
                .map(|x| x.to_le_bytes())
                .collect::<Vec<[u8; 4]>>()
                .concat(),
        )?;
        file.write_all(
            &head_left
                .iter()
                .map(|x| x.to_le_bytes())
                .collect::<Vec<[u8; 4]>>()
                .concat(),
        )?;
        // And now, we go into writing rows.
        let mut row: Vec<u8> = Default::default();
        row.resize((width * 3 + pad) as usize, 0);
        for y in 0..height {
            // populate the row
            for x in 0..width {
                let color = self.get_pixel(x, height - y - 1);
                row[(x * 3) as usize] = color.b;
                row[(x * 3 + 1) as usize] = color.g;
                row[(x * 3 + 2) as usize] = color.r;
            }
            // And write the row.
            file.write_all(&row)?;
        }
        Ok(())
    }
}

// Implementation for cloning a boxed image, this always makes a true copy to a raster image.
impl Clone for Box<dyn Image> {
    fn clone(&self) -> Self {
        return Box::new(RasterImage::new(self.as_ref()));
    }
}

/// Trait to which the desktop frame grabbers adhere.
pub trait Capture {
    /// Capture the frame into an internal buffer, creating a 'snapshot'
    fn capture_image(&mut self) -> bool;

    /// Retrieve the image for access. By default this may be backed by the internal buffer
    /// created by capture_image.
    fn get_image(&mut self) -> Box<dyn Image>;

    /// Retrieve the current full desktop resolution.
    fn get_resolution(&mut self) -> Resolution;

    /// Attempt to prepare capture for a subsection of the entire desktop.
    /// This is implementation defined and not guaranteed to do anything. It MUST be called before
    /// trying to capture an image, as setup may happen here.
    fn prepare_capture(
        &mut self,
        _display: u32,
        _x: u32,
        _y: u32,
        _width: u32,
        _height: u32,
    ) -> bool {
        false
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn test_rgb_order() {
        // Both X11 and Windows use the following to convert from the bytes behind the pointer to
        // the actual pixel values.
        /*
        let masked = as_integer & 0x00FFFFFF;
        RGB {
            r: ((masked >> 16) & 0xFF) as u8,
            g: ((masked >> 8) & 0xFF) as u8,
            b: (masked & 0xFF) as u8,
        }*/
        // Lets make the RGB struct follow that order.
        let as_integer = 0x00112233;
        let masked = as_integer & 0x00FFFFFF;
        let p = RGB {
            r: ((masked >> 16) & 0xFF) as u8,
            g: ((masked >> 8) & 0xFF) as u8,
            b: (masked & 0xFF) as u8,
        };
        assert_eq!(p.r, 0x11);
        assert_eq!(p.g, 0x22);
        assert_eq!(p.b, 0x33);

        // So                 xxRRGGBB
        // let as_integer = 0x00112233;

        // now, we can make an integer, reinterpret cast the thing and check that.
        unsafe {
            let rgb_from_integer =
                std::mem::transmute::<*const i32, *const RGB>(&as_integer as *const i32);
            assert_eq!((*rgb_from_integer).r, 0x11);
            assert_eq!((*rgb_from_integer).g, 0x22);
            assert_eq!((*rgb_from_integer).b, 0x33);
        }
        assert_eq!(std::mem::size_of::<RGB>(), std::mem::size_of::<u32>());
    }
}
