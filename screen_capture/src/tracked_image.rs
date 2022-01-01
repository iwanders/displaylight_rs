//! Wrapper image that allows us to track which pixels were read and how often.

use crate::interface::*;
use crate::raster_image::RasterImage;

enum Event {
    Read(u32, u32),
}

use std::cell::RefCell;

pub struct TrackedImage {
    img: Box<dyn Image>,
    events: RefCell<Vec<Event>>,
}
impl TrackedImage {
    pub fn new(img: Box<dyn Image>) -> TrackedImage {
        TrackedImage {
            img,
            events: RefCell::new(vec![]),
        }
    }

    pub fn draw_access(&self, opacity: f32) -> RasterImage {
        let mut img = RasterImage::new(&*self.img);
        img.scalar_multiply(opacity);
        for event in self.events.borrow_mut().iter() {
            match event {
                Event::Read(x, y) => {
                    let old = self.img.get_pixel(*x, *y);
                    img.set_pixel(
                        *x,
                        *y,
                        RGB {
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

impl Image for TrackedImage {
    fn get_width(&self) -> u32 {
        self.img.get_width()
    }
    fn get_height(&self) -> u32 {
        self.img.get_height()
    }

    fn get_pixel(&self, x: u32, y: u32) -> RGB {
        self.events.borrow_mut().push(Event::Read(x, y));
        self.img.get_pixel(x, y)
    }
}
