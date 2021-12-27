use crate::raster_image::RasterImage;
#[derive(Debug, Default, Copy, Clone)]
pub struct RGB {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

pub trait Image {
    fn get_width(&self) -> u32;
    fn get_height(&self) -> u32;

    fn get_pixel(&self, x: u32, y: u32) -> RGB;

    // Dump a pnm file to disk.
    fn write_pnm(&self, filename: &str) -> std::io::Result<()> {
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

pub trait Grabber {
    fn capture_image(&mut self) -> bool;
    fn get_image(&mut self) -> Box<dyn Image>;
}
