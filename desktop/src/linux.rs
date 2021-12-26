use crate::interface::*;
use libc;

// Here's the X11 ffi part.

// opaque types for pointers.
#[repr(C)]
pub struct Display {
    _private: [u8; 0],
}

#[repr(C)]
pub struct Visual {
    _private: [u8; 0],
}
#[repr(C)]
pub struct Screen {
    _private: [u8; 0],
}

// From X11/X.h
type XID = u64;
type Window = XID;
type Colormap = XID;

#[derive(Debug)]
#[repr(C)]
pub struct XWindowAttributes {
    /* location of window */
    x: i32,
    y: i32,
    /* width and height of window */
    width: i32,
    height: i32,
    /* border width of window */
    border_width: i32,
    /* depth of window */
    depth: i32,
    /* the associated visual structure */
    visual: *mut Visual,
    /* root of screen containing window */
    root: Window,
    /* InputOutput, InputOnly*/
    class: i32,
    /* one of the bit gravity values */
    bit_gravity: i32,
    /* one of the window gravity values */
    win_gravity: i32,
    /* NotUseful, WhenMapped, Always */
    backing_store: i32,
    /* planes to be preserved if possible */
    backing_planes: u64,
    /* value to be used when restoring planes */
    backing_pixel: u64,
    /* boolean, should bits under be saved? */
    save_under: bool,
    /* color map to be associated with window */
    colormap: Colormap,
    /* boolean, is color map currently installed*/
    map_installed: bool,
    /* IsUnmapped, IsUnviewable, IsViewable */
    map_state: i32,
    /* set of events all people have interest in*/
    all_event_masks: i64,
    /* my event mask */
    your_event_mask: i64,
    /* set of events that should not propagate */
    do_not_propagate_mask: i64,
    /* boolean value for override-redirect */
    override_redirect: bool,
    /* back pointer to correct screen */
    screen: *mut Screen,
}
impl Default for XWindowAttributes {
    fn default() -> XWindowAttributes {
        XWindowAttributes {
            x: 0,
            y: 0,
            width: 0,
            height: 0,
            border_width: 0,
            depth: 0,
            visual: 0 as *mut Visual,
            root: 0,
            class: 0,
            bit_gravity: 0,
            win_gravity: 0,
            backing_store: 0,
            backing_planes: 0,
            backing_pixel: 0,
            save_under: false,
            colormap: 0 as Colormap,
            map_installed: false,
            map_state: 0,
            all_event_masks: 0,
            your_event_mask: 0,
            do_not_propagate_mask: 0,
            override_redirect: false,
            screen: 0 as *mut Screen,
        }
    }
}

type Status = i32;

#[link(name = "X11")]
extern "C" {
    fn XOpenDisplay(text: *const libc::c_char) -> *mut Display;

    fn XRootWindow(display: *mut Display, screen_number: i32) -> Window;
    fn XDefaultScreen(display: *mut Display) -> i32;

    fn XGetWindowAttributes(
        display: *mut Display,
        window: Window,
        attributes: *mut XWindowAttributes,
    ) -> Status;
}

#[link(name = "Xext")]
extern "C" {
    fn XShmQueryExtension(display: *mut Display) -> bool;
}

// Then, we can utilise all of that to create an Image instance backed by the shared memory.

struct ImageX11 {}

impl ImageX11 {}

impl Image for ImageX11 {
    fn get_width(&self) -> u32 {
        0
    }
    fn get_height(&self) -> u32 {
        0
    }
}

struct GrabberX11 {
    display: *mut Display,
    window: Window,
}
impl GrabberX11 {
    pub fn new() -> GrabberX11 {
        unsafe {
            let display = XOpenDisplay(0 as *const libc::c_char);
            if (!XShmQueryExtension(display)) {
                panic!("We really need the xshared memory extension. Bailing out.");
            }
            let window = XRootWindow(display, XDefaultScreen(display));
            println!("window: {:?}", window);
            GrabberX11 { display, window }
        }
    }

    pub fn prepare(&mut self, x: u32, y: u32, width: u32, height: u32) {
        let mut attributes = XWindowAttributes::default();
        println!("Attributes: {:?}", attributes);
        println!("self.window: {:?}", self.window);
        let status = unsafe { XGetWindowAttributes(self.display, self.window, &mut attributes) };
        println!("Attributes: {:?}", attributes);

    }
}

impl Grabber for GrabberX11 {
    fn capture_image(&mut self) -> bool {
        true
    }
    fn get_image(&mut self) -> Box<dyn Image> {
        Box::<ImageX11>::new(ImageX11 {})
    }
}

pub fn get_grabber() -> Box<dyn Grabber> {
    let mut z = Box::<GrabberX11>::new(GrabberX11::new());
    z.prepare(0, 0, 1920, 1080);
    z
}
