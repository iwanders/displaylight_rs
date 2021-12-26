pub struct RGB {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

pub trait Image {
    fn get_width(&self) -> u32;
    fn get_height(&self) -> u32;
}

pub trait Grabber {
    fn capture_image(&mut self) -> bool;
    fn get_image(&mut self) -> Box<dyn Image>;
}
