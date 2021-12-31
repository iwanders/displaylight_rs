pub mod border_detection;
pub mod rectangle;
pub mod sampler;
pub mod zones;

use desktop_frame::{get_grabber, Grabber, Resolution};
use lights;
use rectangle::Rectangle;

use std::error::Error;
use std::{thread, time};

use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize, Default, Copy, Clone)]
pub struct CaptureSpecification {
    match_width: Option<u32>,
    match_height: Option<u32>,

    #[serde(default)]
    x: u32,
    #[serde(default)]
    y: u32,

    #[serde(default)]
    width: u32,
    #[serde(default)]
    height: u32,

    #[serde(default)]
    display: u32,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct Config {
    rate: f32,
    port: String,

    vertical_depth: u32,
    horizontal_depth: u32,

    sample_pixel_distance: u32,

    edge_detection_bisect_count: u32,

    limiting_factor: f32,

    capture: Vec<CaptureSpecification>,
}

// Iterates through the specs to find the best one, augmends the missing or 0 values and returns it.
fn get_config(width: u32, height: u32, specs: &[CaptureSpecification]) -> CaptureSpecification {
    for spec in specs.iter() {
        let mut matches = true;
        if let Some(match_width) = spec.match_width {
            matches &= match_width == width;
        }
        if let Some(match_height) = spec.match_height {
            matches &= match_height == height;
        }
        if !matches {
            continue;
        }

        // We found the best match, copy this and populate it as best we can.
        let mut populated: CaptureSpecification = *spec;
        populated.width = if populated.width == 0 {
            width - populated.x
        } else {
            populated.width
        };
        populated.height = if populated.height == 0 {
            height - populated.y
        } else {
            populated.height
        };
        return populated;
    }

    // No capture match found... well, return some sane default then.
    CaptureSpecification {
        width,
        height,
        ..Default::default()
    }
}

pub struct DisplayLight {
    config: Config,
    grabber: Box<dyn Grabber>,
    lights: lights::Lights,
}

impl DisplayLight {
    const MAX_LEDS: usize = 228;

    pub fn new(config: Config) -> Result<DisplayLight, Box<dyn Error>> {
        Ok(DisplayLight {
            config: config,
            grabber: desktop_frame::get_grabber(),
            lights: lights::Lights::new("/dev/ttyACM0")?,
        })
    }

    fn setup(&mut self) {
        self.lights.set_limit_factor(self.config.limiting_factor);
    }

    pub fn run(&mut self) -> Result<(), Box<dyn Error>> {
        // Perform one time setup.
        self.setup();

        let mut canvas = [lights::RGB::default(); DisplayLight::MAX_LEDS];

        let mut cached_sampler: Option<(Rectangle, sampler::Sampler)> = None;
        let mut cached_resolution: Option<Resolution> = None;
        loop {
            // First, check if the resolution of the desktop environment has changed, if so, act.
            let current_resolution = self.grabber.get_resolution();
            if cached_resolution.is_none()
                || *cached_resolution.as_ref().unwrap() != current_resolution
            {
                let width = current_resolution.width;
                let height = current_resolution.height;

                // Resolution has changed, figure out the best match in our configurations and
                // prepare the capture accordingly.
                let config = get_config(width, height, &self.config.capture);

                self.grabber.prepare_capture(
                    config.display,
                    config.x,
                    config.y,
                    config.width,
                    config.height,
                );
            }

            // Now, we are ready to try and get the image:
            let res = self.grabber.capture_image();
            if !res {
                // Getting the image failed... :( Lets wait a bit and try again.
                thread::sleep(time::Duration::from_millis(10));
                continue;
            }

            // Then, we can grab the actual image.
            let img = self.grabber.get_image();

            // Detect the black borders
            let borders =
                border_detection::find_borders(&*img, self.config.edge_detection_bisect_count);

            // Border size changed, make a new sampler.
            if cached_sampler.is_none() || cached_sampler.as_ref().unwrap().0 != borders {
                // With the edges known, we can make the zones.
                let zones = zones::Zones::make_zones(
                    &borders,
                    self.config.horizontal_depth,
                    self.config.vertical_depth,
                );
                // println!("zones: {:?}", zones);
                assert_eq!(zones.len(), DisplayLight::MAX_LEDS);

                // With the zones known, we can create the sampler.
                let sampler =
                    sampler::Sampler::make_sampler(&zones, self.config.sample_pixel_distance);
                cached_sampler = Some((borders, sampler));
            }

            // With the sampler, we can now sample and get color values.
            let sampler = &cached_sampler.as_ref().unwrap().1;
            sampler.sample_into(&*img, &mut canvas);

            // And, finally, we can set the leds to those colors.
            self.lights.set_leds(&canvas)?;
            thread::sleep(time::Duration::from_millis(1000 / 60));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use desktop_frame::raster_image::make_dummy_gradient;
    use desktop_frame::raster_image::RasterImage;
    use desktop_frame::{Image, RGB};
    use std::env::temp_dir;

    fn tmp_file(name: &str) -> String {
        temp_dir()
            .join(name)
            .to_str()
            .expect("path must be ok")
            .to_owned()
    }

    #[test]
    fn test_full() {
        // Make a dummy image.
        let img = make_dummy_gradient();
        img.write_bmp(
            temp_dir()
                .join("gradient.bmp")
                .to_str()
                .expect("path must be ok"),
        )
        .unwrap();

        // Detect the black borders
        let mut tracked = desktop_frame::tracked_image::TrackedImage::new(Box::new(img));
        let b = border_detection::find_borders(&tracked, 5);
        let mut track_results = tracked.draw_access(0.5);
        track_results.set_pixel(b.x_min, b.y_min, RGB::cyan());
        track_results.set_pixel(b.x_max, b.y_max, RGB::white());
        track_results
            .write_ppm(&tmp_file("test_full_borders.ppm"))
            .expect("Should succeed.");

        // With the edges known, we can make the zones.
        let zones = zones::Zones::make_zones(&b, 100, 100);
        assert_eq!(zones.len(), 228);

        // With the zones known, we can create the sampler.
        let sampler = sampler::Sampler::make_sampler(&zones, 10);

        // With the sampler, we can now sample and get color values.
        tracked.clear_events();
        let values = sampler.sample(&tracked);
        assert_eq!(values.len(), 228);
        let mut track_results = tracked.draw_access(0.5);

        track_results
            .write_ppm(&tmp_file("test_full_sampling.ppm"))
            .expect("Should succeed.");

        // With the values known, we can color the zones appropriately.
        let mut canvas = RasterImage::filled(
            tracked.get_width(),
            tracked.get_height(),
            Default::default(),
        );
        for (i, zone) in zones.iter().enumerate() {
            canvas.fill_rectangle(zone.x_min, zone.x_max, zone.y_min, zone.y_max, values[i])
        }
        canvas
            .write_bmp(
                temp_dir()
                    .join("analysed_canvas.bmp")
                    .to_str()
                    .expect("path must be ok"),
            )
            .unwrap();
    }

    #[test]
    fn test_config() {
        let spec1: CaptureSpecification = CaptureSpecification {
            match_width: Some(3840),
            x: 1920,
            ..Default::default()
        };
        let spec2: CaptureSpecification = CaptureSpecification {
            ..Default::default()
        };
        let specs = [spec1, spec2];
        let res = get_config(3840, 1080, &specs);
        assert_eq!(res.x, 1920);
        assert_eq!(res.width, 3840 - 1920);
        assert_eq!(res.height, 1080);
    }
}
