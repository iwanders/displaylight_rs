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
