use screen_capture::raster_image::RasterImageBGR;
use screen_capture::{ImageBGR, BGR};

pub const WHITE: BGR = BGR {
    r: 255,
    g: 255,
    b: 255,
};

pub const YELLOW: BGR = BGR {
    r: 0,
    g: 255,
    b: 255,
};

pub const CYAN: BGR = BGR {
    r: 0,
    g: 255,
    b: 255,
};

enum Event {
    Read(u32, u32),
}

use std::cell::RefCell;

pub struct TrackedImage {
    img: Box<dyn ImageBGR>,
    events: RefCell<Vec<Event>>,
}
impl TrackedImage {
    pub fn new(img: Box<dyn ImageBGR>) -> TrackedImage {
        TrackedImage {
            img,
            events: RefCell::new(vec![]),
        }
    }

    pub fn draw_access(&self, opacity: f32) -> RasterImageBGR {
        let mut img = RasterImageBGR::new(&*self.img);
        img.scalar_multiply(opacity);
        for event in self.events.borrow_mut().iter() {
            match event {
                Event::Read(x, y) => {
                    let old = self.img.pixel(*x, *y);
                    img.set_pixel(
                        *x,
                        *y,
                        BGR {
                            r: 255,
                            g: std::cmp::min(255, old.g as u32 + 20) as u8,
                            b: std::cmp::min(255, old.b as u32 + 20) as u8,
                        },
                    );
                }
            }
        }
        img
    }

    pub fn clear_events(&self) {
        self.events.borrow_mut().clear()
    }
}

impl ImageBGR for TrackedImage {
    fn width(&self) -> u32 {
        self.img.width()
    }
    fn height(&self) -> u32 {
        self.img.height()
    }

    fn pixel(&self, x: u32, y: u32) -> BGR {
        self.events.borrow_mut().push(Event::Read(x, y));
        self.img.pixel(x, y)
    }
    fn data(&self) -> &[BGR] {
        panic!()
    }
}
