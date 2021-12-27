use crate::raster_image::RasterImage;

#[derive(Debug, Default, Copy, Clone)]
/// Struct to represent a single pixel.
pub struct RGB {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

#[derive(Debug, Default, Copy, Clone)]
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

    /// Dump a pnm file to disk.
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
            file.write(v.as_ref())?;
            file.write(b"\n")?;
        }
        Ok(())
    }
}

impl Clone for Box<dyn Image> {
    fn clone(&self) -> Self {
        return Box::new(RasterImage::new(self.as_ref()));
    }
}

/// Trait to which the desktop frame grabbers adhere.
pub trait Grabber {
    /// Capture the frame into an internal buffer, creating a 'snapshot'
    fn capture_image(&mut self) -> bool;

    /// Retrieve the image for access. By default this may be backed by the internal buffer
    /// created by capture_image.
    fn get_image(&mut self) -> Box<dyn Image>;

    /// Retrieve the current full desktop resolution.
    fn get_resolution(&mut self) -> Resolution;

    /// Attempt to prepare capture for a subsection of the entire desktop.
    fn prepare_capture(&mut self, _x: u32, _y: u32, _width: u32, _height: u32) -> bool {
        return false;
    }
}
