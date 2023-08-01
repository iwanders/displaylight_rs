use crate::interface::*;
mod X11;
use X11::*;

mod shm;

/// Image wrapper around XImage.
struct ImageX11 {
    image: Option<*mut XImage>,
}

impl ImageX11 {}

impl Image for ImageX11 {
    fn get_width(&self) -> u32 {
        if self.image.is_none() {
            panic!("Used get_width on an image that doesn't exist.");
        }
        unsafe { (*self.image.unwrap()).width as u32 }
    }
    fn get_height(&self) -> u32 {
        if self.image.is_none() {
            panic!("Used get_width on an image that doesn't exist.");
        }
        unsafe { (*self.image.unwrap()).height as u32 }
    }

    fn get_pixel(&self, x: u32, y: u32) -> RGB {
        if self.image.is_none() {
            panic!("no image present to retrieve pixel");
        }
        let width = self.get_width();
        let height = self.get_height();
        if x > width || y > height {
            panic!("Retrieved out of bounds ({}, {})", x, y);
        }

        unsafe {
            let image = &(*(self.image.unwrap()));
            // println!("Image: {:?}", self.image.unwrap());
            // Do some pointer magic and reach into the data, do a few casts and we're golden.
            let data = std::mem::transmute::<*const libc::c_char, *const u8>(image.data);
            let stride = (image.bits_per_pixel / 8) as u32;
            let as_integer = *std::mem::transmute::<*const u8, *const u32>(
                data.offset((y * width * stride + x * stride).try_into().unwrap()),
            );
            let masked = as_integer & 0x00FFFFFF;
            RGB {
                r: ((masked >> 16) & 0xFF) as u8,
                g: ((masked >> 8) & 0xFF) as u8,
                b: (masked & 0xFF) as u8,
            }
        }
    }

    fn get_data(&self) -> Option<&[RGB]> {
        if self.image.is_none() {
            return None; // we can fail gracefully, might as well.
        }
        unsafe {
            let image = &(*(self.image.unwrap()));
            let width = image.width as usize;
            let height = image.height as usize;
            assert!(image.bits_per_pixel / 8 == 4);
            let data = std::mem::transmute::<*const libc::c_char, *const RGB>(image.data);
            let len = width * height;
            Some(std::slice::from_raw_parts(data, len))
        }
    }
}

/// Capture struct for X11.
struct CaptureX11 {
    display: *mut Display,
    window: Window,
    image: Option<*mut XImage>,
    shminfo: XShmSegmentInfo,
    pos_x: u32,
    pos_y: u32,
}

impl Drop for CaptureX11 {
    fn drop(&mut self) {
        // Clean up the memory correctly.
        unsafe {
            if self.image.is_some() {
                XDestroyImage(self.image.unwrap());
            }
        }
    }
}

impl CaptureX11 {
    pub fn new() -> CaptureX11 {
        unsafe {
            let display = XOpenDisplay(std::ptr::null::<libc::c_char>());
            if XShmQueryExtension(display) == 0 {
                panic!("We really need the xshared memory extension. Bailing out.");
            }
            let window = XRootWindow(display, XDefaultScreen(display));
            CaptureX11 {
                display,
                window,
                image: None,
                shminfo: Default::default(),
                pos_x: 0,
                pos_y: 0,
            }
        }
    }
    pub fn prepare(&mut self, x: u32, y: u32, width: u32, height: u32) -> bool {
        let mut attributes = XWindowAttributes::default();
        let status = unsafe { XGetWindowAttributes(self.display, self.window, &mut attributes) };
        if status != 1 {
            panic!("Retrieving the window attributes failed.");
        }

        let width = std::cmp::min(
            if width != 0 {
                width as i32
            } else {
                attributes.width
            },
            attributes.width,
        );
        let height = std::cmp::min(
            if height != 0 {
                height as i32
            } else {
                attributes.height
            },
            attributes.height,
        );

        let x = std::cmp::min(x as i32, attributes.width);
        let y = std::cmp::min(y as i32, attributes.height);
        self.pos_x = x as u32;
        self.pos_y = y as u32;

        let width = std::cmp::min(width, attributes.width - x as i32);
        let height = std::cmp::min(height, attributes.height - y as i32);

        self.image = Some(unsafe {
            XShmCreateImage(
                self.display,
                attributes.visual,
                attributes.depth as u32,
                ZPixmap,
                std::ptr::null_mut::<libc::c_char>(),
                &mut self.shminfo,
                width as u32,
                height as u32,
            )
        });

        let ximage = self.image.unwrap();
        // Next, create the shared memory information.
        unsafe {
            self.shminfo.shmid = shm::shmget(
                shm::IPC_PRIVATE,
                ((*ximage).bytes_per_line * (*ximage).height) as u64,
                shm::IPC_CREAT | 0x180,
            );

            (*ximage).data = std::mem::transmute::<*mut libc::c_void, *mut libc::c_char>(
                shm::shmat(self.shminfo.shmid, std::ptr::null_mut::<libc::c_void>(), 0),
            );
            self.shminfo.shmaddr = (*ximage).data;
            self.shminfo.readOnly = 0;

            // And now, we just have to attach the shared memory.
            if XShmAttach(self.display, &self.shminfo) == 0 {
                panic!("Couldn't attach shared memory");
            }
        }
        true
    }
}

impl Capture for CaptureX11 {
    fn capture_image(&mut self) -> bool {
        if self.image.is_none() {
            return false;
        }
        let z;

        unsafe {
            z = XShmGetImage(
                self.display,
                self.window,
                self.image.unwrap(),
                self.pos_x as i32,
                self.pos_y as i32,
                AllPlanes,
            );
        }
        z
    }
    fn get_image(&mut self) -> Box<dyn Image> {
        if self.image.is_some() {
            Box::<ImageX11>::new(ImageX11 {
                image: Some(self.image.unwrap()),
            })
        } else {
            Box::<ImageX11>::new(ImageX11 { image: None })
        }
    }

    fn get_resolution(&mut self) -> Resolution {
        let mut x: i32 = 0;
        let mut y: i32 = 0;
        let mut width: u32 = 0;
        let mut height: u32 = 0;
        let mut border_width: u32 = 0;
        let mut depth: u32 = 0;
        let mut window: Window = Default::default();
        unsafe {
            XGetGeometry(
                self.display,
                self.window,
                &mut window,
                &mut x,
                &mut y,
                &mut width,
                &mut height,
                &mut border_width,
                &mut depth,
            );
        }

        Resolution { width, height }
    }

    fn prepare_capture(&mut self, _display: u32, x: u32, y: u32, width: u32, height: u32) -> bool {
        CaptureX11::prepare(self, x, y, width, height)
    }
}

unsafe extern "C" fn error_handler(_display: *mut Display, event: *mut XErrorEvent) -> i32 {
    println!("Error: {:?}", event);
    0
}

pub fn get_capture() -> Box<dyn Capture> {
    unsafe {
        XSetErrorHandler(error_handler);
    }
    let mut z = Box::<CaptureX11>::new(CaptureX11::new());
    z.prepare(0, 0, 0, 0);
    z
}
