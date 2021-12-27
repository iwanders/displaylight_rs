pub mod interface;
pub mod raster_image;

use crate::interface::*;

#[cfg_attr(target_os = "linux", path = "./linux/linux.rs")]
#[cfg_attr(windows, path = "windows.rs")]
mod backend;

/// Get a new instance of the desktop frame grabber for this platform.
pub fn get_grabber() -> Box<dyn Grabber> {
    return backend::get_grabber();
}
