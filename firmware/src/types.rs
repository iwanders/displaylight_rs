#[repr(C, packed)]
#[derive(Default, Copy, Clone)]
/// Struct to represent the RGB state of a single led.
pub struct RGB {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}
impl RGB {
    pub const RED: RGB = RGB { r: 255, g: 0, b: 0 };
    pub const GREEN: RGB = RGB { r: 0, g: 255, b: 0 };
    pub const BLUE: RGB = RGB { r: 0, g: 0, b: 255 };
    pub const WHITE: RGB = RGB {
        r: 255,
        g: 255,
        b: 255,
    };
    pub const BLACK: RGB = RGB { r: 0, g: 0, b: 0 };

    pub fn limit(&mut self, limit: u8) {
        self.r = core::cmp::min(self.r, limit);
        self.g = core::cmp::min(self.g, limit);
        self.b = core::cmp::min(self.b, limit);
    }
}
