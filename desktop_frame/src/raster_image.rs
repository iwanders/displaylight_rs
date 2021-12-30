use crate::interface::*;

#[derive(Default)]
pub struct RasterImage {
    data: Vec<Vec<RGB>>,
}
impl RasterImage {
    pub fn new(img: &dyn Image) -> RasterImage {
        let mut res: RasterImage = Default::default();
        let width = img.get_width();
        let height = img.get_height();
        res.data.resize(height as usize, Default::default());
        for y in 0..height {
            res.data[y as usize].resize(width as usize, Default::default());
            for x in 0..width {
                res.data[y as usize][x as usize] = img.get_pixel(x, y);
            }
        }
        return res;
    }

    pub fn filled(width: u32, height: u32, color: RGB) -> RasterImage {
        let mut res: RasterImage = Default::default();
        res.data.resize(height as usize, Default::default());
        for y in 0..height {
            res.data[y as usize].resize(width as usize, Default::default());
            for x in 0..width {
                res.data[y as usize][x as usize] = color;
            }
        }
        res
    }

    pub fn fill_rectangle(&mut self, x_min: u32, x_max: u32, y_min: u32, y_max: u32, color: RGB) {
        for y in y_min..y_max{
            for x in x_min..x_max{
                self.set_pixel(x, y, color);
            }
        }
    }

    pub fn from_2d_vec(data: &Vec<Vec<RGB>>) -> RasterImage {
        RasterImage {
            data: data.to_vec(),
        }
    }

    pub fn set_pixel(&mut self, x: u32, y: u32, color: RGB) {
        let width = self.get_width();
        let height = self.get_height();
        if x > width || y > height {
            panic!("Trying to set out of bounds ({}, {})", x, y);
        }
        self.data[y as usize][x as usize] = color;
    }

    // Ugly gradient for visual inspection.
    pub fn set_gradient(&mut self, x_min: u32, x_max: u32, y_min: u32, y_max: u32) {
        let r_step = 255.0 / (x_max - x_min) as f64;
        let g_step = 255.0 / (y_max - y_min) as f64;
        for y in y_min..y_max {
            for x in x_min..x_max {
                self.set_pixel(
                    x,
                    y,
                    RGB {
                        r: (((x - x_min) as f64 * r_step) as u32 % 256) as u8,
                        g: (((y - y_min) as f64 * g_step) as u32 % 256) as u8,
                        b: 255 - (((x - x_min) as f64 * r_step) as u32 % 256) as u8,
                    },
                );
            }
        }
    }
}

impl Image for RasterImage {
    fn get_width(&self) -> u32 {
        if self.data.len() == 0 {
            return 0;
        }
        return self.data[0].len().try_into().unwrap();
    }
    fn get_height(&self) -> u32 {
        return self.data.len().try_into().unwrap();
    }
    fn get_pixel(&self, x: u32, y: u32) -> RGB {
        return self.data[y as usize][x as usize];
    }
}

// Mostly for testing...
pub fn make_dummy_gradient() -> RasterImage
{
    let mut img = RasterImage::filled(1920, 1080, RGB { r: 0, g: 0, b: 0 });
    img.set_gradient(200, 1920 - 200, 0, 1080);
    img
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use std::env::temp_dir;

    #[test]
    fn test_draw_gradient() {
        let mut img = RasterImage::filled(100, 100, RGB { r: 0, g: 0, b: 0 });
        img.set_gradient(10, 90, 20, 80);
        img.write_bmp(
            temp_dir()
                .join("gradient.bmp")
                .to_str()
                .expect("path must be ok"),
        )
        .unwrap();
        let img = make_dummy_gradient();
        img.write_bmp(
            temp_dir()
                .join("gradient_big.bmp")
                .to_str()
                .expect("path must be ok"),
        )
        .unwrap();
    }
}
