pub mod interface;
pub mod raster_image;

pub use interface::{Grabber, Image, RGB};

use crate::interface::*;

#[cfg_attr(target_os = "linux", path = "./linux/linux.rs")]
#[cfg_attr(target_os = "windows", path = "./windows/windows.rs")]
mod backend;

/// Get a new instance of the desktop frame grabber for this platform.
pub fn get_grabber() -> Box<dyn Grabber> {
    return backend::get_grabber();
}

/// Reads a pnm image from disk. (or rather pnms written by `Image::write_pnm`).
pub fn read_ppm(filename: &str) -> Result<Box<dyn Image>, Box<dyn std::error::Error>> {
    println!("zzz: {:?}", "dsfsdf");
    use std::fs::File;
    let file = File::open(filename)?;
    use std::io::{BufRead, BufReader};
    let br = BufReader::new(file);
    let mut lines = br.lines();
    let width: u32;
    let height: u32;
    fn make_error(v: &str) -> Box<dyn std::error::Error> {
        Box::new(std::io::Error::new(std::io::ErrorKind::Other, v))
    }

    // First, read the type, this must be P3
    let l = lines.next().ok_or(make_error("Not enough lines"))??;
    if l != "P3" {
        return Err(make_error("Input format not supported."));
    }

    // This is where we get the resolution.
    let l = lines.next().ok_or(make_error("Not enough lines"))??;
    let mut values = l.trim().split(" ").map(|x| str::parse::<u32>(x));
    width = values
        .next()
        .ok_or(make_error("Could not parse width."))??;
    height = values
        .next()
        .ok_or(make_error("Could not parse height."))??;

    // And check the scaling.
    let l = lines.next().ok_or(make_error("Not enough lines"))??;
    if l != "255" {
        return Err(make_error("Scaling not supported, only 255 supported"));
    }

    let mut img: Vec<Vec<RGB>> = Default::default();
    img.resize(height as usize, vec![]);

    // Now, we iterate over the remaining lines, each holds a row for the image.
    for (li, l) in lines.enumerate() {
        let l = l?;
        // Allocate this row.
        img[li].resize(width as usize, Default::default());
        // Finally, parse the row.
        // https://doc.rust-lang.org/rust-by-example/error/iter_result.html
        let z = l.trim().split(" ").map(|x| str::parse::<u32>(x));
        let numbers: Result<Vec<_>, _> = z.collect();
        let numbers = numbers?;
        // Cool, now we have a bunch of numbers, verify the width.
        if numbers.len() / 3 != width as usize {
            return Err(make_error(
                format!("Width is incorrect, got {}", numbers.len() / 3).as_str(),
            ));
        }

        // Finally, we can convert the bytes.
        for i in 0..width as usize {
            let r = u8::try_from(numbers[i * 3 + 0])?;
            let g = u8::try_from(numbers[i * 3 + 1])?;
            let b = u8::try_from(numbers[i * 3 + 2])?;
            img[li][i] = RGB { r, g, b };
        }
    }

    return Ok(Box::new(raster_image::RasterImage::from_2d_vec(&img)));
}
