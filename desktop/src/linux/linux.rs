use crate::interface::*;
mod X11;
use X11::*;

mod shm;

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
    image: Option<*mut XImage>,
}

impl Drop for GrabberX11 {
    fn drop(&mut self) {
        // Clean up the memory correctly.
        unsafe {
            if (self.image.is_some()) {
                XDestroyImage(self.image.unwrap());
            }
        }
    }
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
            println!("display: {:?}", display);
            GrabberX11 {
                display,
                window,
                image: None,
            }
        }
    }
    pub fn prepare(&mut self, x: u32, y: u32, width: u32, height: u32) {
        let mut attributes = XWindowAttributes::default();
        println!("self.window: {:?}", self.window);
        let status = unsafe { XGetWindowAttributes(self.display, self.window, &mut attributes) };
        println!("Attributes: {:#?}", attributes);

        let width = std::cmp::min(if (width != 0) {width as i32} else {attributes.width}, attributes.width);
        let height = std::cmp::min(if (height != 0) {height as i32} else {attributes.height}, attributes.height);

        let x = std::cmp::min(x as i32, attributes.width);
        let y = std::cmp::min(y as i32, attributes.height);

        let width = std::cmp::min(width, attributes.width - x as i32);
        let height = std::cmp::min(height, attributes.height - y as i32);

        let mut shminfo: XShmSegmentInfo = Default::default();
        self.image = Some(unsafe {
            XShmCreateImage(
                self.display,
                attributes.visual,
                attributes.depth as u32,
                ZPixmap,
                0 as *mut libc::c_char,
                &mut shminfo,
                width as u32,
                height as u32,
            )
        });
        let ximage = self.image.unwrap();
        // Next, create the shared memory information.
        unsafe {
            println!("ximage; {:#?}", *ximage);
            println!("shminfo; {:#?}", shminfo);
            shminfo.shmid = shm::shmget(
                shm::IPC_PRIVATE,
                ((*ximage).bytes_per_line * (*ximage).height) as u64,
                shm::IPC_CREAT | 0x180,
            );
            println!("shminfo; {:#?}", shminfo);
            (*ximage).data = std::mem::transmute::<*mut libc::c_void, *mut libc::c_char>(
                shm::shmat(shminfo.shmid, 0 as *const libc::c_void, 0)
            );
            shminfo.shmaddr = (*ximage).data;
            shminfo.readOnly = false;
            println!("shminfo; {:#?}", shminfo);
            println!("(*ximage).data; {:#?}", (*ximage).data);

            // And now, we just have to attach the shared memory.
            if (!XShmAttach(self.display, &shminfo)) {
                panic!("Couldn't attach shared memory");
            }
            println!("shminfo; {:#?}", shminfo);
        }
    }
}

impl Grabber for GrabberX11 {
    fn capture_image(&mut self) -> bool {
        if (self.image.is_none()) {
            return false;
        }
        let z;
        println!("going into XShmGetImage");
        unsafe {
            z = XShmGetImage(
                self.display,
                self.window,
                self.image.unwrap(),
                0,
                0,
                AllPlanes,
            );
        }
        println!("z: {:?}", z);
        return z;
    }
    fn get_image(&mut self) -> Box<dyn Image> {
        Box::<ImageX11>::new(ImageX11 {})
    }
}

pub fn get_grabber() -> Box<dyn Grabber> {
    let mut z = Box::<GrabberX11>::new(GrabberX11::new());
    z.prepare(0, 0, 0, 0);
    z
}
