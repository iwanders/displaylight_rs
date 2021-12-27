pub mod interface;
pub mod raster_image;
use crate::interface::*;

#[cfg_attr(target_os = "linux", path = "./linux/linux.rs")]
#[cfg_attr(windows, path = "windows.rs")]
mod backend;

pub fn get_grabber() -> Box<dyn Grabber> {
    return backend::get_grabber();
}
