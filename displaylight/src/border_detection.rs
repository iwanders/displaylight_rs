// Roughly the same architecture as the C++ project.
// Analyser does bisection to find the black borders.
// Then sample in each led's rectangle.

use desktop_frame::{Image, RGB};
use crate::rectangle::{Rectangle};

// This bespoke bisection procedure to find the presumably single transition in a 1d search.
fn bisect(f: &dyn Fn(u32) -> bool, min: &mut u32, max: &mut u32) {
    let mut upper = f(*min); // dummy values.
    let mut lower = f(*max);
    while (*max - *min >= 2) && (upper != lower) {
        upper = f(*max);
        lower = f(*min);
        let midpoint = (*max + *min) / 2;
        let center = f(midpoint);
        if center != lower {
            *max = midpoint
        } else {
            *min = midpoint
        }
    }
}

pub fn find_borders(image: &dyn Image, bisections_per_side: u32) -> Rectangle {
    let mut b: Rectangle = Default::default();
    use std::cmp::{max, min};

    // No idea if this is the fastest way to write it... but it is cool.
    let bounds = (0..bisections_per_side)
        .map(|i| {
            let mut bisection_res: [u32; 4] = [0, image.get_width() - 1, 0, image.get_height() - 1];
            let mut tmp;
            let mid_x = (image.get_width() - 1) / (bisections_per_side + 1) * (i + 1);
            let mid_y = (image.get_height() - 1) / (bisections_per_side + 1) * (i + 1);

            // Perform left bound, find x_min
            tmp = mid_x;
            bisect(
                &|x| image.get_pixel(x, mid_y) != RGB::black(),
                &mut bisection_res[0],
                &mut tmp,
            );

            // Perform right bound, find x_max
            tmp = mid_x;
            bisect(
                &|x| image.get_pixel(x, mid_y) != RGB::black(),
                &mut tmp,
                &mut bisection_res[1],
            );

            // Perform lower bound, find y_min
            tmp = mid_y;
            bisect(
                &|y| image.get_pixel(mid_x, y) != RGB::black(),
                &mut bisection_res[2],
                &mut tmp,
            );

            // Perform upper bound, find y_max
            tmp = mid_y;
            bisect(
                &|y| image.get_pixel(mid_x, y) != RGB::black(),
                &mut tmp,
                &mut bisection_res[3],
            );

            // println!("Bisection res: {:?}", bisection_res);
            return bisection_res;
        })
        .reduce(|a, b| {
            [
                max(a[0], b[0]),
                min(a[1], b[1]),
                max(a[2], b[2]),
                min(a[3], b[3]),
            ]
        });

    let bounds = bounds.expect("Will always have a result.");
    b.x_min = bounds[0];
    b.x_max = bounds[1];
    b.y_min = bounds[2];
    b.y_max = bounds[3];
    b
}

#[cfg(test)]
mod tests {
    use super::*;
    use desktop_frame::raster_image::RasterImage;
    use desktop_frame::Image;

    #[test]
    fn test_bisect() {
        let z = [0u32, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        // Bisect to find the first index that is leq 5.
        let mut min_v = 0u32;
        let mut max_v = z.len() as u32 - 1;
        bisect(&|i| z[i as usize] <= 5, &mut min_v, &mut max_v);
        assert_eq!(min_v, 5);
        assert_eq!(max_v, 6);

        let z = [0u32, 0, 0, 0, 4, 4, 4, 4, 4, 4, 4];
        min_v = 0u32;
        max_v = z.len() as u32 - 1;
        bisect(&|i| z[i as usize] != 0, &mut min_v, &mut max_v);
        assert_eq!(min_v, 3);
        assert_eq!(max_v, 4);
    }

    #[test]
    fn test_fully_white() {
        let img = RasterImage::filled(
            100,
            100,
            RGB {
                r: 255,
                g: 255,
                b: 255,
            },
        );
        let b = find_borders(&img, 5);

        assert_eq!(b.x_min, 0);
        assert_eq!(b.y_min, 0);
        assert_eq!(b.x_max, 99);
        assert_eq!(b.y_max, 99);
    }
    #[test]
    fn test_real() {
        let mut img = RasterImage::filled(100, 100, RGB { r: 0, g: 0, b: 0 });
        for y in 20..70u32 {
            for x in 30..80u32 {
                img.set_pixel(
                    x,
                    y,
                    RGB {
                        r: 255,
                        g: 255,
                        b: 0,
                    },
                );
            }
        }
        let b = find_borders(&img, 6);
        // img.write_bmp("/tmp/foo.bmp");

        assert_eq!(b.x_min, 29);
        assert_eq!(b.y_min, 19);
        assert_eq!(b.x_max, 80);
        assert_eq!(b.y_max, 70);
    }
}
