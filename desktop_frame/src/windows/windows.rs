use crate::interface::*;

struct ImageWin {
}

impl ImageWin {}

impl Image for ImageWin {
    fn get_width(&self) -> u32 {
        0
    }
    fn get_height(&self) -> u32 {
        0
    }
    fn get_pixel(&self, x: u32, y: u32) -> RGB {
        RGB {
            r: 0,
            g: 0,
            b: 0,
        }
    }
}



struct GrabberWin {
}

impl Drop for GrabberWin {
    fn drop(&mut self) {
    }
}

impl GrabberWin {
    pub fn new() -> GrabberWin {
        GrabberWin {}
    }
    pub fn prepare(&mut self, x: u32, y: u32, width: u32, height: u32) -> bool {
        true
    }
}

impl Grabber for GrabberWin {
    fn capture_image(&mut self) -> bool {
        false
    }
    fn get_image(&mut self) -> Box<dyn Image> {
            Box::<ImageWin>::new(ImageWin {})
    }

    fn get_resolution(&mut self) -> Resolution {
        Resolution { width: 0, height: 0 }
    }

    fn prepare_capture(&mut self, x: u32, y: u32, width: u32, height: u32) -> bool {
        return GrabberWin::prepare(self, x, y, width, height);
    }
}

pub fn get_grabber() -> Box<dyn Grabber> {
    let mut z = Box::<GrabberWin>::new(GrabberWin::new());
    z.prepare(0, 0, 0, 0);
    z
}
