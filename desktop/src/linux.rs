use crate::interface::*;
use libc;

// Here's the X11 ffi part.

// opaque types for pointers.
#[repr(C)]
pub struct Display {
    _private: [u8; 0],
}

#[repr(C)]
pub struct Window {
    _private: [u8; 0],
}

#[link(name = "X11")]
extern "C" {
    fn XOpenDisplay(text: *const libc::c_char) -> *mut Display;
    fn XRootWindow() -> *mut Window;
}

#[link(name = "Xext")]
extern "C" {
    fn XShmQueryExtension(display: *mut Display) -> bool;
}


// Then, we can utilise all of that to create an Image instance backed by the shared memory.

struct ImageX11
{
    display: *mut Display,
    window: *mut Window,
}

impl ImageX11
{
    pub fn new() -> ImageX11
    {
          // display_ = XOpenDisplay(nullptr);
          // root_window_ = XRootWindow(display_, XDefaultScreen(display_));

          // if (!XShmQueryExtension(display_))
          // {
            // throw std::runtime_error("XShmQueryExtension needs to be available.");
          // }
        unsafe {
            let display = XOpenDisplay(0 as *const libc::c_char);
            if (!XShmQueryExtension(display))
            {
                panic!("We really need the xshared memory extension. Bailing out.");
            }
            let window = XRootWindow();
            ImageX11{display, window}
        }
    }
}

impl Image for ImageX11
{
    fn get_width(&self) -> u32
    {
        0
    }
    fn get_height(&self) -> u32
    {
        0
    }
}

struct GrabberX11
{
}

impl Grabber for GrabberX11
{
    fn capture_image(&mut self) -> bool
    {
        true
    }
    fn get_image(&mut self) -> Box<dyn Image>
    {
        Box::<ImageX11>::new(ImageX11::new())
    }
}


pub fn get_grabber() -> Box<dyn Grabber>
{
    Box::<GrabberX11>::new(GrabberX11{})
    
}