//! A struct that efficiently samples the image and calculates averaged values.
use crate::rectangle::Rectangle;
use desktop_frame::{Image, RGB};
use lights::RGB as lRGB;

#[derive(Copy, Clone, Debug)]
struct Index {
    pub x: u32,
    pub y: u32,
}

/// Sampler struct that precomputes the indices to sample on.
pub struct Sampler {
    indices: Vec<Vec<Index>>,
}

impl Sampler {
    /// Make a sampler that's ready to 
    pub fn make_sampler(zones: &[Rectangle], distance_between_samples: u32) -> Sampler {
        // Prepares indices for sampling.
        let mut sampler: Sampler = Sampler { indices: vec![] };
        sampler.indices.resize(zones.len(), vec![]);

        // iterate over the zones.
        for (i, zone) in zones.iter().enumerate() {
            // Sample from the center of the zones.
            let height = std::cmp::min(1, (zone.y_max - zone.y_min) / distance_between_samples + 1);
            let width = std::cmp::min(1, (zone.x_max - zone.x_min) / distance_between_samples + 1);
            sampler.indices[i].reserve((height * width) as usize);

            // In a nice equidistant grid.
            for y in (zone.y_min..zone.y_max).step_by(distance_between_samples as usize) {
                for x in (zone.x_min..zone.x_max).step_by(distance_between_samples as usize) {
                    sampler.indices[i].push(Index { x, y });
                }
            }
        }
        sampler
    }

    // Sample an image and return a vector of RGB values.
    pub fn sample(&self, image: &dyn Image) -> Vec<RGB> {
        // Use the prepared indices for sampling, going from an image to a set of colors.
        let mut res: Vec<RGB> = Vec::<RGB>::with_capacity(self.indices.len());
        res.resize(self.indices.len(), Default::default());
        for (i, sample_points) in self.indices.iter().enumerate() {
            // Do something smart here like collecting all pixels on the sample points...
            let mut r = 0u32;
            let mut g = 0u32;
            let mut b = 0u32;
            let mut t = 0u32;
            for point in sample_points.iter() {
                let pixel = image.get_pixel(point.x, point.y);
                r += pixel.r as u32;
                g += pixel.g as u32;
                b += pixel.b as u32;
                t += 1;
            }
            // This shouldn't every happen, but lets handle it in case there's no sample points in
            // the cell.
            if t == 0 {
                res[i] = RGB::black();
                continue;
            }
            res[i] = RGB {
                r: (r / t) as u8,
                g: (g / t) as u8,
                b: (b / t) as u8,
            };
        }
        res
    }

    /// Sample an image and write the results into an array of [`lights::RGB`].
    pub fn sample_into(&self, image: &dyn Image, res: &mut [lRGB]) {
        // Use the prepared indices for sampling, going from an image to a set of colors.
        for (i, sample_points) in self.indices.iter().enumerate() {
            // Do something smart here like collecting all pixels on the sample points...
            let mut r = 0u32;
            let mut g = 0u32;
            let mut b = 0u32;
            let mut t = 0u32;
            for point in sample_points.iter() {
                let pixel = image.get_pixel(point.x, point.y);
                r += pixel.r as u32;
                g += pixel.g as u32;
                b += pixel.b as u32;
                t += 1;
            }

            // This shouldn't every happen, but lets handle it in case there's no sample points in
            // the cell.
            if t == 0 {
                res[i].r = 0;
                res[i].g = 0;
                res[i].b = 0;
                continue;
            }
            res[i].r = (r / t) as u8;
            res[i].g = (g / t) as u8;
            res[i].b = (b / t) as u8;
        }
    }
}
