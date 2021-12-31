pub mod border_detection;
pub mod rectangle;
pub mod sampler;
pub mod zones;

use rectangle::Rectangle;
use lights;

use std::error::Error;
use std::{thread, time};

use serde::{Serialize, Deserialize};


#[derive(Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct CaptureSpecification
{
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
pub struct Config
{
    rate: f32,
    port: String,

    vertical_depth: u32,
    horizontal_depth: u32,

    sample_pixel_distance: u32,

    edge_detection_bisect_count: u32,

    limiting_factor: f32,

    capture: Vec<CaptureSpecification>,
}

pub struct DisplayLight
{
    config: Config,
}

impl DisplayLight
{
    const MAX_LEDS: usize = 228;

    fn run(&mut self) -> Result<(), Box<dyn Error>>
    {
        let mut grabber = desktop_frame::get_grabber();

        let resolution = grabber.get_resolution();

        println!("Grabber reports resolution of: {:?}", resolution);
        grabber.prepare_capture(1920, 0, resolution.width - 1920, resolution.height);

        let mut control = lights::Lights::new("/dev/ttyACM0")?;
        control.set_limit_factor(0.5);

        let mut canvas = [lights::RGB::default(); DisplayLight::MAX_LEDS];

        let mut state: Option<(Rectangle, sampler::Sampler)> = None;
        loop {
            let res = grabber.capture_image();
            if !res {
                continue;
            }
            // Then, grab the image.
            let img = grabber.get_image();

            // Detect the black borders
            let borders = border_detection::find_borders(&*img, 4);

            // Border size changed, make a new sampler.
            if state.is_none() || state.as_ref().unwrap().0 != borders {
                // With the edges known, we can make the zones.
                let zones = zones::Zones::make_zones(&borders, 200, 200);
                // println!("zones: {:?}", zones);
                assert_eq!(zones.len(), DisplayLight::MAX_LEDS);

                // With the zones known, we can create the sampler.
                let sampler = sampler::Sampler::make_sampler(&zones, 15);
                state = Some((borders, sampler));
            }

            let sampler = &state.as_ref().unwrap().1;
            // With the sampler, we can now sample and get color values.
            sampler.sample_into(&*img, &mut canvas);

            control.set_leds(&canvas)?;
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
}
