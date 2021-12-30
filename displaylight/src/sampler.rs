use crate::rectangle::Rectangle;
use desktop_frame::{Image, RGB};

#[derive(Copy, Clone)]
struct Index {
    pub x: u32,
    pub y: u32,
}

pub struct Sampler {
    indices: Vec<Vec<Index>>,
}

impl Sampler {
    pub fn make_sampler(zones: &[Rectangle]) -> Sampler {
        // Prepares indices for sampling.
        let mut sampler: Sampler = Sampler { indices: vec![] };
        sampler.indices.resize(zones.len(), vec![]);
        // Sample center point for now.
        for (i, zone) in zones.iter().enumerate() {
            sampler.indices[i].push(Index {
                x: (zone.x_max - zone.x_min) / 2 + zone.x_min,
                y: (zone.y_max - zone.y_min) / 2 + zone.y_min,
            });
        }
        sampler
    }

    pub fn sample(&self, image: &dyn Image) -> Vec<RGB> {
        // Use the prepared indices for sampling, going from an image to a set of colors.
        let mut res: Vec<RGB> = Vec::<RGB>::with_capacity(self.indices.len());
        res.resize(self.indices.len(), Default::default());
        for (i, sample_points) in self.indices.iter().enumerate() {
            // Do something smart here like collecting all pixels on the sample points...
            let sample_pos = sample_points[0];
            res[i] = image.get_pixel(sample_pos.x, sample_pos.y);
        }
        res
    }
}
