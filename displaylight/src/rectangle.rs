/// Represents a rectangle on a grid.
#[derive(Debug, Default, Eq, PartialEq, Copy, Clone)]
pub struct Rectangle {
    pub x_min: u32,
    pub x_max: u32,
    pub y_min: u32,
    pub y_max: u32,
}
