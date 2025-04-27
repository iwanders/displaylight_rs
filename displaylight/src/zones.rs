//! Definition of the zones we can individually control on the led strip. Basically, this is the
//! mapping between physical leds and regions on the screen.

use crate::rectangle::Rectangle;

pub struct Zones {}

impl Zones {
    // Numbers from the C++ code, they pertain to the physical led mapping.
    const LEDS: u32 = 228;
    const HORIZONTAL: u32 = 73;
    const VERTICAL: u32 = 42;

    /// Make the zones for the provided rectangle, horizontal depth and vertical depth.
    pub fn make_zones(
        rectangle: &Rectangle,
        horizontal_depth: u32,
        vertical_depth: u32,
    ) -> Vec<Rectangle> {
        let mut res: Vec<Rectangle> = Vec::with_capacity(Zones::LEDS as usize);

        let width = rectangle.x_max - rectangle.x_min;
        let height = rectangle.y_max - rectangle.y_min;

        // Inclusive bounds:
        // left side 0 - 41 (starts top)
        // bottom side: 42 - 113 (starts left)
        // right side: 114 - 155 (starts bottom)
        // top side: 156 - 227 (starts right)
        let vertical_step: u32 = height / Zones::VERTICAL;
        let horizontal_step: u32 = width / Zones::HORIZONTAL;

        // Ugly code follows, copied from the C++ side.
        for led in 0..Zones::LEDS {
            let (x_min, x_max, y_min, y_max);
            if led < 42 {
                let pos = led;
                // left side.
                x_min = rectangle.x_min;
                x_max = rectangle.x_min + horizontal_depth;
                y_min = rectangle.y_min + pos * vertical_step;
                y_max = rectangle.y_min + (pos + 1) * vertical_step;
                res.push(Rectangle {
                    x_min,
                    x_max,
                    y_min,
                    y_max,
                });
            } else if led < 114 {
                // bottom
                let pos = led - 42;
                x_min = rectangle.x_min + pos * horizontal_step;
                x_max = rectangle.x_min + (pos + 1) * horizontal_step;
                y_min = rectangle.y_min + height - vertical_depth;
                y_max = rectangle.y_min + height;
                res.push(Rectangle {
                    x_min,
                    x_max,
                    y_min,
                    y_max,
                });
            } else if led < 156 {
                // right side.
                let pos = led - 114;
                x_min = rectangle.x_min + width - horizontal_depth;
                x_max = rectangle.x_min + width;
                y_min = rectangle.y_min + height - (pos + 1) * vertical_step;
                y_max = rectangle.y_min + height - (pos) * vertical_step;
                res.push(Rectangle {
                    x_min,
                    x_max,
                    y_min,
                    y_max,
                });
            } else if led < Zones::LEDS + 1 {
                // top side
                let pos = led - 156;
                x_min = rectangle.x_min + width - (pos + 1) * horizontal_step;
                x_max = rectangle.x_min + width - (pos) * horizontal_step;
                y_min = rectangle.y_min;
                y_max = rectangle.y_min + vertical_depth;
                res.push(Rectangle {
                    x_min,
                    x_max,
                    y_min,
                    y_max,
                });
            }
        }
        res
    }
}
