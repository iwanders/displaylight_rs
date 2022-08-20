//! Find the borders that bound the non black region in an image.

use crate::rectangle::Rectangle;
use screen_capture::{Image, RGB};

// This bespoke bisection procedure to find the presumably single transition in a 1d search.
// This bails out if lower and upper are identical, so if the return of f at start min and max
// is identical, it will return max if f(max) was true, else it returns min.
fn bisect(f: &dyn Fn(u32) -> bool, min: u32, max: u32) -> u32 {
    let mut min = min;
    let mut max = max;
    let mut upper = f(min);
    let mut lower = f(max);
    while ((max - min) > 1) && (upper != lower) {
        upper = f(max);
        lower = f(min);
        let midpoint = (max + min) / 2;
        let center = f(midpoint);
        if center != lower {
            max = midpoint
        } else {
            min = midpoint
        }
    }

    if upper {
        return max;
    }

    min
}

/// find the borders that define the useful region in this image.
///
/// * `bisections_per_side` - The number of bisections to perform per side.
/// * `only_rectangular` - If true, only returns a Some if the bisecions agreed on proper rectangle with straight edges.
pub fn find_borders(
    image: &dyn Image,
    bisections_per_side: u32,
    only_rectangular: bool,
) -> Option<Rectangle> {
    let mut b: Rectangle = Default::default();
    use std::cmp::{max, min};

    // No idea if this is the fastest way to write it... but it is cool with the reduce.
    // Notice the lambda changes between ==black and != black, this ensures that in a completely
    // black situation, we pick the correct side to return.

    let mut transitions: [u32; 4] = [0; 4];
    let bounds = (0..bisections_per_side)
        .map(|i| {
            let mut bisection_res: [u32; 4] = [0, 0, 0, 0];
            let max_x = image.get_width() - 1;
            let max_y = image.get_height() - 1;
            let center_x = max_x / 2;
            let center_y = max_y / 2;
            let mid_x = max_x / (bisections_per_side + 1) * (i + 1);
            let mid_y = max_y / (bisections_per_side + 1) * (i + 1);

            // Perform left bound, find x_min
            bisection_res[0] = bisect(&|x| image.get_pixel(x, mid_y) == RGB::black(), 0, center_x);

            // Perform right bound, find x_max
            bisection_res[1] = bisect(
                &|x| image.get_pixel(x, mid_y) != RGB::black(),
                center_x,
                max_x,
            );

            // Perform lower bound, find y_min
            bisection_res[2] = bisect(&|y| image.get_pixel(mid_x, y) == RGB::black(), 0, center_y);

            // Perform upper bound, find y_max
            bisection_res[3] = bisect(
                &|y| image.get_pixel(mid_x, y) != RGB::black(),
                center_y,
                max_y,
            );

            // println!("Bisection res: {:?}", bisection_res);
            bisection_res
        })
        .reduce(|a, b| {
            for i in 0..4 {
                if a[i] != b[i] {
                    transitions[i] += 1;
                }
            }
            [
                min(a[0], b[0]),
                max(a[1], b[1]),
                min(a[2], b[2]),
                max(a[3], b[3]),
            ]
        });

    // println!("transitions res: {:?}", transitions);
    // Any more than 4 transitions means we have something that's not rectangular.
    if only_rectangular && (transitions.iter().reduce(|a, b| max(a, b)).unwrap() >= &4) {
        return None;
    }

    let bounds = bounds.expect("Will always have a result.");
    // For x_min and y_min, add one if the alue is not zero, this ensures we start on the non-white
    // boundary. This does make it a bit odd if we actually have a bisection result that would
    // truly be x_min=0, but in all other cases this means we start on the correct pixel where the
    // non-black starts.
    b.x_min = if bounds[0] != 0 {
        bounds[0] + 1
    } else {
        bounds[0]
    };
    b.x_max = bounds[1];
    b.y_min = if bounds[2] != 0 {
        bounds[2] + 1
    } else {
        bounds[2]
    };
    b.y_max = bounds[3];

    // But that causes problems if the image is completely black, as x_min then exceeds x_max.
    // So here we fix that by ensuring x_min <= x_max, and y_min <= y_max.
    if b.x_min > b.x_max {
        b.x_min = b.x_max
    }
    if b.y_min > b.y_max {
        b.y_min = b.y_max
    }

    Some(b)
}

#[derive(Debug, Clone, Copy)]
/// Struct to smoothly rate limit rectangle size changes.
pub struct RectangleChangeLimiter {
    x_min: f32,
    x_max: f32,
    y_min: f32,
    y_max: f32,

    previous_time: std::time::Instant,
    horizontal_rate_per_s: f32,
    vertical_rate_per_s: f32,
}

impl RectangleChangeLimiter {
    /// Instantiate a new limiter with a horizontal rate limit and vertical rate limit.
    pub fn new(horizontal_rate_per_s: f32, vertical_rate_per_s: f32) -> Self {
        RectangleChangeLimiter {
            x_min: 0.0,
            x_max: 0.0,
            y_min: 0.0,
            y_max: 0.0,
            previous_time: std::time::Instant::now(),
            horizontal_rate_per_s,
            vertical_rate_per_s,
        }
    }

    /// Set the current rectangle without rate limiting.
    pub fn set(&mut self, rectangle: &Rectangle, current: &std::time::Instant) {
        self.x_min = rectangle.x_min as f32;
        self.x_max = rectangle.x_max as f32;
        self.y_min = rectangle.y_min as f32;
        self.y_max = rectangle.y_max as f32;
        self.previous_time = *current;
    }

    /// Update the rectangle with rate limiting.
    pub fn update(&mut self, rectangle: &Rectangle, current: &std::time::Instant) {
        let dt = (*current - self.previous_time).as_secs_f32();
        self.previous_time = *current;

        let dh = dt * self.horizontal_rate_per_s;
        let dv = dt * self.vertical_rate_per_s;

        self.x_min = self.x_min + (rectangle.x_min as f32 - self.x_min).clamp(-dh, dh);
        self.x_max = self.x_max + (rectangle.x_max as f32 - self.x_max).clamp(-dh, dh);

        self.y_min = self.y_min + (rectangle.y_min as f32 - self.y_min).clamp(-dv, dv);
        self.y_max = self.y_max + (rectangle.y_max as f32 - self.y_max).clamp(-dv, dv);
    }

    /// Return the current rectangle.
    pub fn rectangle(&self) -> Rectangle {
        Rectangle {
            x_min: self.x_min as u32,
            x_max: self.x_max as u32,
            y_min: self.y_min as u32,
            y_max: self.y_max as u32,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use screen_capture::raster_image::RasterImage;
    use screen_capture::Image;
    use std::env::temp_dir;

    fn tmp_file(name: &str) -> String {
        temp_dir()
            .join(name)
            .to_str()
            .expect("path must be ok")
            .to_owned()
    }

    #[test]
    fn test_bisect() {
        //       0     1  2  3  4  5  6  7  8  9
        let z = [0u32, 0, 0, 0, 0, 1, 1, 1, 1, 1];
        let len = z.len() as u32 - 1;
        // Bisect to find the first index that is leq 5.
        let res = bisect(&|i| z[i as usize] == 0, 0u32, len);
        assert_eq!(res, 4);

        let res = bisect(&|i| z[i as usize] != 0, 0u32, len);
        assert_eq!(res, 5);

        let rz = z.iter().rev().collect::<Vec<&u32>>();
        let res = bisect(&|i| *rz[i as usize] == 0, 0u32, len);
        assert_eq!(res, 5);
        let res = bisect(&|i| *rz[i as usize] != 0, 0u32, len);
        assert_eq!(res, 4);

        // Completely black.
        let v = [0u32, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];

        // Bias low
        let res = bisect(&|i| v[i as usize] != 0, 0u32, len);
        assert_eq!(res, 0);

        // Bias high.
        let res = bisect(&|i| v[i as usize] == 0, 0u32, len);
        assert_eq!(res, len);

        // Completely white.
        let v = [1u32, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1];

        // Bias low
        let res = bisect(&|i| v[i as usize] == 0, 0u32, len);
        assert_eq!(res, 0);

        // Bias high.
        let res = bisect(&|i| v[i as usize] != 0, 0u32, len);
        assert_eq!(res, len);
    }

    #[test]
    fn test_fully_white() {
        let img = RasterImage::filled(100, 100, RGB::white());
        let b = find_borders(&img, 5, false).expect("Only rectangular is false.");

        assert_eq!(b.x_min, 0);
        assert_eq!(b.y_min, 0);
        assert_eq!(b.x_max, 99);
        assert_eq!(b.y_max, 99);
    }

    #[test]
    fn test_free_floating_rect() {
        let mut img = RasterImage::filled(100, 100, RGB { r: 0, g: 0, b: 0 });
        img.fill_rectangle(30, 80, 20, 70, RGB::yellow());
        let tracked = screen_capture::tracked_image::TrackedImage::new(Box::new(img));
        let b = find_borders(&tracked, 10, false).expect("Only rectangular is false.");
        let mut track_results = tracked.draw_access(0.5);
        track_results.set_pixel(b.x_min, b.y_min, RGB::cyan());
        track_results.set_pixel(b.x_max, b.y_max, RGB::white());
        track_results
            .write_ppm(&tmp_file("free_floating.ppm"))
            .expect("Should succeed.");

        assert_eq!(b.x_min, 30); // last index that is black
        assert_eq!(b.y_min, 20); // last index that is black.
        assert_eq!(b.x_max, 79); // last index that is not black.
        assert_eq!(b.y_max, 69); // last index that is not black.
    }

    #[test]
    fn test_horizontal_borders() {
        let mut img = RasterImage::filled(100, 100, RGB { r: 0, g: 0, b: 0 });
        img.fill_rectangle(0, 100, 20, 70, RGB::yellow());
        let tracked = screen_capture::tracked_image::TrackedImage::new(Box::new(img));
        let b = find_borders(&tracked, 10, false).expect("Only rectangular is false.");
        let mut track_results = tracked.draw_access(0.5);
        track_results.set_pixel(b.x_min, b.y_min, RGB::cyan());
        track_results.set_pixel(b.x_max, b.y_max, RGB::white());
        track_results
            .write_ppm(&tmp_file("test_horizontal_borders.ppm"))
            .expect("Should succeed.");

        assert_eq!(b.x_min, 0); // last index that is black
        assert_eq!(b.y_min, 20); // last index that is black.
        assert_eq!(b.x_max, 99); // last index that is not black.
        assert_eq!(b.y_max, 69); // last index that is not black.
    }

    #[test]
    fn test_vertical_borders() {
        let mut img = RasterImage::filled(100, 100, RGB { r: 0, g: 0, b: 0 });
        img.fill_rectangle(30, 80, 0, 100, RGB::yellow());
        let tracked = screen_capture::tracked_image::TrackedImage::new(Box::new(img));
        let b = find_borders(&tracked, 10, false).expect("Only rectangular is false.");
        let mut track_results = tracked.draw_access(0.5);
        track_results.set_pixel(b.x_min, b.y_min, RGB::cyan());
        track_results.set_pixel(b.x_max, b.y_max, RGB::white());
        track_results
            .write_ppm(&tmp_file("test_vertical_borders.ppm"))
            .expect("Should succeed.");

        assert_eq!(b.x_min, 30); // last index that is black
        assert_eq!(b.y_min, 0); // last index that is black.
        assert_eq!(b.x_max, 79); // last index that is not black.
        assert_eq!(b.y_max, 99); // last index that is not black.
    }

    #[test]
    fn test_black() {
        let img = RasterImage::filled(1920, 1080, RGB { r: 0, g: 0, b: 0 });
        let tracked = screen_capture::tracked_image::TrackedImage::new(Box::new(img));
        let b = find_borders(&tracked, 10, false).expect("Only rectangular is false.");
        let mut track_results = tracked.draw_access(0.5);
        track_results.set_pixel(b.x_min, b.y_min, RGB::cyan());
        track_results.set_pixel(b.x_max, b.y_max, RGB::white());
        track_results
            .write_bmp(&tmp_file("test_black.bmp"))
            .expect("Should succeed.");
        // println!("Borders: {:?}", b);
        assert_eq!(b.x_min, 959); // last index that is black
        assert_eq!(b.y_min, 539); // last index that is black.
        assert_eq!(b.x_max, 959); // last index that is not black.
        assert_eq!(b.y_max, 539); // last index that is not black.
    }

    #[test]
    fn test_only_rectangular() {
        let mut img = RasterImage::filled(100, 100, RGB { r: 0, g: 0, b: 0 });
        img.fill_rectangle(20, 60, 20, 60, RGB::yellow());
        img.fill_rectangle(40, 80, 40, 80, RGB::yellow());
        let tracked = screen_capture::tracked_image::TrackedImage::new(Box::new(img));
        let b = find_borders(&tracked, 10, true);
        let track_results = tracked.draw_access(0.5);
        track_results
            .write_ppm(&tmp_file("test_only_rectangular.ppm"))
            .expect("Should succeed.");
        assert!(b.is_none());

        let mut img = RasterImage::filled(100, 100, RGB { r: 0, g: 0, b: 0 });
        img.fill_rectangle(10, 40, 30, 60, RGB::yellow());
        img.fill_rectangle(30, 70, 20, 30, RGB::yellow());
        img.fill_rectangle(60, 90, 30, 60, RGB::yellow());
        let tracked = screen_capture::tracked_image::TrackedImage::new(Box::new(img));
        let b = find_borders(&tracked, 10, true);
        let track_results = tracked.draw_access(0.5);
        track_results
            .write_ppm(&tmp_file("test_only_rectangular2.ppm"))
            .expect("Should succeed.");
        assert!(b.is_none());
    }

    #[test]
    fn test_rectangle_limiter() {
        let mut z = RectangleChangeLimiter::new(10.0, 15.0);
        let init = Rectangle {
            x_min: 0,
            x_max: 100,
            y_min: 0,
            y_max: 100,
        };
        let t0 = std::time::Instant::now();
        z.set(&init, &t0);

        // Elapse one second
        let t1 = t0 + std::time::Duration::from_secs_f32(1.0);
        let reduced = Rectangle {
            x_min: 20,
            x_max: 80,
            y_min: 20,
            y_max: 80,
        };
        // Update, rate limit should happen with 1s.
        z.update(&reduced, &t1);

        let r = z.rectangle();
        assert_eq!(r.x_min, 10);
        assert_eq!(r.x_max, 90);
        assert_eq!(r.y_min, 15);
        assert_eq!(r.y_max, 85);

        // Elapse another second, now it should be fully matching the reduced rectangle.
        let t2 = t1 + std::time::Duration::from_secs_f32(1.0);
        z.update(&reduced, &t2);
        let r = z.rectangle();
        assert_eq!(r, reduced);

        // Grow the rectangle in size for one second.
        let t3 = t2 + std::time::Duration::from_secs_f32(1.0);
        let increased = Rectangle {
            x_min: 0,
            x_max: 120,
            y_min: 0,
            y_max: 120,
        };
        z.update(&increased, &t3);
        let r = z.rectangle();
        assert_eq!(r.x_min, 10);
        assert_eq!(r.x_max, 90);
        assert_eq!(r.y_min, 5);
        assert_eq!(r.y_max, 95);

        let t4 = t3 + std::time::Duration::from_secs_f32(1.0);
        let t5 = t4 + std::time::Duration::from_secs_f32(1.0);
        let t6 = t5 + std::time::Duration::from_secs_f32(1.0);
        z.update(&increased, &t6);
        let r = z.rectangle();
        assert_eq!(r, increased);
    }
}
