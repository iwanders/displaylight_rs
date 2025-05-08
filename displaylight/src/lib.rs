//! A crate to set leds to the same color as regions on the screen.
//!
//! The following happens in a loop:
//!   - Retrieval of the image shown on the screen.
//!   - Black border detection, if we have black borders we want to ignore this and get colors from the interesting part.
//!   - Sample regions associated to each led.
//!   - Set the leds to the average of the sampled colors.
//!   - Sleep to ensure we match a certain update interval.
//!
//! What also happens is that if the resolution changes, the capture can be reconfigured based on a
//! priority list, this allows retrieving a specific monitor if there's a multi monitor setup.

pub mod border_detection;
pub mod rate_limiter;
pub mod rectangle;
pub mod sampler;
pub mod zones;

#[cfg(test)]
pub mod test_util;

use rectangle::Rectangle;
use screen_capture::{Capture, Resolution};

use serde::{Deserialize, Serialize};
use std::error::Error;

/// Capture specification, if `match_*` is populated and matches the resolution's value it will be
/// considered to match and the capture will be setup according to the other fields.
#[derive(Debug, PartialEq, Serialize, Deserialize, Default, Copy, Clone)]
pub struct CaptureSpecification {
    /// The resolution's width to match to.
    pub match_width: Option<u32>,

    /// The resolution's height to match to.
    pub match_height: Option<u32>,

    #[serde(default)]
    /// The x offset to apply for this specification.
    pub x: u32,
    /// The y offset to apply for this specification.
    #[serde(default)]
    pub y: u32,

    /// The width to apply for this specification, set to the resolutions' width - x if zero.
    #[serde(default)]
    pub width: u32,
    /// The height to apply for this specification, set to the resolutions' height - y if zero.
    #[serde(default)]
    pub height: u32,

    /// The display to set the capture setup to.
    #[serde(default)]
    pub display: u32,
}

/// Configuration struct, specifying all the configurable properties of the displaylight struct..
#[derive(Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct Config {
    /// The update rate at which the loop should run in Hz.
    pub rate: f32,

    /// The serial port path or name used to control the leds. Like "/dev/ttyACM0" or "COM5".
    pub port: String,

    /// The depth in pixels of the vertical cells at the top and bottom of the screen.
    pub vertical_depth: u32,

    /// The depth in pixels of the horizontal cells at the left and right of the screen.
    pub horizontal_depth: u32,

    /// The distance between sampled pixels in the cells.
    pub sample_pixel_distance: u32,

    /// Whether or not to diagonalize the points to be sampled. This attempts to avoid the sample
    /// points making horizontal and vertical lines.
    pub sample_diagonalize_points: bool,

    /// The number of bisections to perform on each frame's side to determine the bounds.
    pub edge_detection_bisect_count: u32,

    /// Only change detection rectangle if the detected borders are rectangular.
    pub edge_detection_rectangular_only: bool,

    /// If false, always use the full width and height of the image.
    pub edge_detection_enable: bool,

    /// Allowed edge change (pixels) in horizontal direction per second.
    pub edge_horizontal_change_per_s: f32,

    /// Allowed edge change (pixels) in vertical direction per second.
    pub edge_vertical_change_per_s: f32,

    /// The limiting factor for the overall led brightness.
    pub limiting_factor: f32,

    /// A list of capture specifications, the first one to match will be used.
    pub capture: Vec<CaptureSpecification>,
}

/// Iterates through the specs to find the best one, augmends the missing or 0 values and returns it.
/// See the documentation of [`CaptureSpecification`] for further information.
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

/// DisplayLight object that will perform the loop to check the screen, analyse and update the leds.
pub struct DisplayLight {
    config: Config,
    grabber: Option<Box<dyn Capture>>,
    lights: lights::Lights,
    limiter: rate_limiter::Limiter,
}

impl DisplayLight {
    const MAX_LEDS: usize = 228;

    /// Instantiate a new instance using the provided configuration. This will try to connect to
    /// the serial port immediately and returns failure if that doesn't succeed.
    pub fn new(config: Config) -> Result<DisplayLight, Box<dyn Error>> {
        Ok(DisplayLight {
            limiter: rate_limiter::Limiter::new(config.rate),
            lights: lights::Lights::new(&config.port)?,
            config,
            grabber: None,
        })
    }

    fn setup(&mut self) {
        self.lights.set_limit_factor(self.config.limiting_factor);
    }

    /// Enter the main loop, this function will never return.
    pub fn run(&mut self) -> Result<(), Box<dyn Error>> {
        // Perform one time setup.
        self.setup();

        // Create the canvas, container of current led pixels to be updated or reused.
        let mut canvas = [lights::RGB::default(); DisplayLight::MAX_LEDS];

        // Sampler only updates based on the black border detection, cache it such that we can reuse
        // it.
        let mut cached_sampler: Option<(Rectangle, sampler::Sampler)> = None;

        // Border change rate limiter, to avoid flickering.
        let mut border_rate_limiter = border_detection::RectangleChangeLimiter::new(
            self.config.edge_horizontal_change_per_s,
            self.config.edge_vertical_change_per_s,
        );

        // The resolution is used for the capture setup and config retrieval, store the old value.
        let mut cached_resolution: Option<Resolution> = None;

        let mut consecutive_capture_fails: usize = 0;
        loop {
            // If the grabber isn't setup yet, try to set it up.
            if self.grabber.is_none() {
                let grabber = screen_capture::capture();
                // Ensure we also clear the cached resolution, such that we actually prepare the capture again.
                cached_resolution = None;
                match grabber {
                    Ok(g) => self.grabber = Some(g),
                    Err(e) => {
                        println!("Setting up grabber failed: {e:?}");
                        self.limiter.sleep();
                        continue;
                    }
                }
            }
            let grabber = self.grabber.as_mut().unwrap();

            // First, check if the resolution of the desktop environment has changed, if so, act.
            let current_resolution = grabber.resolution();
            if cached_resolution.is_none()
                || *cached_resolution.as_ref().unwrap() != current_resolution
            {
                let width = current_resolution.width;
                let height = current_resolution.height;

                // Resolution has changed, figure out the best match in our configurations and
                // prepare the capture accordingly.
                let config = get_config(width, height, &self.config.capture);

                if let Err(e) = grabber.prepare_capture(
                    config.display,
                    config.x,
                    config.y,
                    config.width,
                    config.height,
                ) {
                    println!("Failed preparing capture {e:?}");
                    self.grabber = None;
                    continue;
                };
                // Store the current resolution.
                cached_resolution = Some(current_resolution);
            }

            // Now, we are ready to try and get the image:
            let res = grabber.capture_image();
            if let Err(e) = res {
                consecutive_capture_fails += 1;
                if consecutive_capture_fails > 10 {
                    println!("Got 10 consecutive capture fails, resetting grabber; {e:?}");
                    self.grabber = None;
                    consecutive_capture_fails = 0;
                }
                // Getting the image failed... :( Lets wait a bit and try again.
                // Lets keep the leds at the old color. May make failures less noticable, but uac on windows doesn't
                // look ugly when we can't grab the image for a while.
                self.lights.set_leds(&canvas)?;
                self.limiter.sleep();
                continue;
            }

            // Then, we can grab the actual image.
            let img = grabber.image();
            if let Err(e) = img {
                self.lights.set_leds(&canvas)?;
                self.limiter.sleep();
                consecutive_capture_fails += 1;
                println!("Failed to retrieve {consecutive_capture_fails} images, error: {e:?}");
                continue;
            }
            consecutive_capture_fails = 0;
            let img = img.unwrap();

            // Detect the black borders if we are configured to do so.
            let borders = if self.config.edge_detection_enable {
                border_detection::find_borders(
                    &*img,
                    self.config.edge_detection_bisect_count,
                    self.config.edge_detection_rectangular_only,
                )
            } else {
                Some(Rectangle {
                    x_min: 0,
                    y_min: 0,
                    x_max: img.width() - 1,
                    y_max: img.height() - 1,
                })
            };

            // Border size changed, make a new sampler.
            if let Some(mut borders) = borders {
                // First update, force the border rate change.
                if cached_sampler.is_none() {
                    border_rate_limiter.set(&borders, &std::time::Instant::now());
                }
                border_rate_limiter.update(&borders, &std::time::Instant::now());
                borders = border_rate_limiter.rectangle();

                if cached_sampler.is_none() || cached_sampler.as_ref().unwrap().0 != borders {
                    // println!("Borders: {:?}", borders);
                    // With the edges known, we can make the zones.
                    let zones = zones::Zones::make_zones(
                        &borders,
                        self.config.horizontal_depth,
                        self.config.vertical_depth,
                    );
                    // println!("zones: {:?}", zones);
                    assert_eq!(zones.len(), DisplayLight::MAX_LEDS);

                    // With the zones known, we can create the sampler.
                    let sampler = sampler::Sampler::make_sampler(
                        &zones,
                        self.config.sample_pixel_distance,
                        self.config.sample_diagonalize_points,
                    );
                    cached_sampler = Some((borders, sampler));
                }
            }

            // With the sampler, we can now sample and get color values.
            let sampler = &cached_sampler.as_ref().unwrap().1;
            sampler.sample_into(&*img, &mut canvas);

            // And, finally, we can set the leds to those colors.
            self.lights.set_leds(&canvas)?;
            self.limiter.sleep();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::{CYAN, WHITE};
    use screen_capture::raster_image::RasterImageBGR;
    use screen_capture::{ImageBGR, BGR};
    use std::env::temp_dir;

    fn tmp_file(name: &str) -> String {
        temp_dir()
            .join(name)
            .to_str()
            .expect("path must be ok")
            .to_owned()
    }

    fn make_dummy_gradient() -> RasterImageBGR {
        let mut img = RasterImageBGR::filled(1920, 1080, BGR { r: 0, g: 0, b: 0 });
        img.set_gradient(200, 1920 - 200, 0, 1080);
        img
    }
    #[test]
    fn test_full() {
        use screen_capture::util::WriteSupport;
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
        let tracked = crate::test_util::TrackedImage::new(Box::new(img));
        let b =
            border_detection::find_borders(&tracked, 5, false).expect("Only rectangular is false");
        let mut track_results = tracked.draw_access(0.5);
        track_results.set_pixel(b.x_min, b.y_min, CYAN);
        track_results.set_pixel(b.x_max, b.y_max, WHITE);
        track_results
            .write_ppm(&tmp_file("test_full_borders.ppm"))
            .expect("Should succeed.");

        // With the edges known, we can make the zones.
        let zones = zones::Zones::make_zones(&b, 200, 200);
        assert_eq!(zones.len(), 228);

        // With the zones known, we can create the sampler.
        let sampler = sampler::Sampler::make_sampler(&zones, 15, true);

        // With the sampler, we can now sample and get color values.
        tracked.clear_events();
        let values = sampler.sample(&tracked);
        assert_eq!(values.len(), 228);
        let track_results = tracked.draw_access(0.5);
        let values: Vec<BGR> = values
            .iter()
            .copied()
            .map(|z| BGR {
                r: z.r,
                g: z.g,
                b: z.b,
            })
            .collect();

        track_results
            .write_ppm(&tmp_file("test_full_sampling.ppm"))
            .expect("Should succeed.");

        // With the values known, we can color the zones appropriately.
        let mut canvas =
            RasterImageBGR::filled(tracked.width(), tracked.height(), Default::default());
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
