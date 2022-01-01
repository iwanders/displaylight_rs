#![allow(
    non_camel_case_types,
    non_snake_case,
    non_upper_case_globals,
    dead_code
)]
#![allow(clippy::upper_case_acronyms)]
// Minimal Rust bindings for the X11 parts we need. Implemented from the X11 headers which are
// Licensed under the following license.
/*

Copyright 1985, 1986, 1987, 1991, 1998  The Open Group

Permission to use, copy, modify, distribute, and sell this software and its
documentation for any purpose is hereby granted without fee, provided that
the above copyright notice appear in all copies and that both that
copyright notice and this permission notice appear in supporting
documentation.

The above copyright notice and this permission notice shall be included in
all copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT.  IN NO EVENT SHALL THE
OPEN GROUP BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN
AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN
CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.

Except as contained in this notice, the name of The Open Group shall not be
used in advertising or otherwise to promote the sale, use or other dealings
in this Software without prior written authorization from The Open Group.

*/

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
pub type Window = XID;
pub type Drawable = XID;
pub type Colormap = XID;

type Bool = i32; // Wow!?

#[derive(Debug)]
#[repr(C)]
pub struct XWindowAttributes {
    /* location of window */
    pub x: i32,
    pub y: i32,
    /* width and height of window */
    pub width: i32,
    pub height: i32,
    /* border width of window */
    pub border_width: i32,
    /* depth of window */
    pub depth: i32,
    /* the associated visual structure */
    pub visual: *mut Visual,
    /* root of screen containing window */
    pub root: Window,
    /* InputOutput, InputOnly*/
    pub class: i32,
    /* one of the bit gravity values */
    pub bit_gravity: i32,
    /* one of the window gravity values */
    pub win_gravity: i32,
    /* NotUseful, WhenMapped, Always */
    pub backing_store: i32,
    /* planes to be preserved if possible */
    pub backing_planes: u64,
    /* value to be used when restoring planes */
    pub backing_pixel: u64,
    /* boolean, should bits under be saved? */
    pub save_under: Bool,
    /* color map to be associated with window */
    pub colormap: Colormap,
    /* boolean, is color map currently installed*/
    pub map_installed: Bool,
    /* IsUnmapped, IsUnviewable, IsViewable */
    pub map_state: i32,
    /* set of events all people have interest in*/
    pub all_event_masks: i64,
    /* my event mask */
    pub your_event_mask: i64,
    /* set of events that should not propagate */
    pub do_not_propagate_mask: i64,
    /* boolean value for override-redirect */
    pub override_redirect: Bool,
    /* back pointer to correct screen */
    pub screen: *mut Screen,
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
            visual: std::ptr::null_mut::<Visual>(),
            root: 0,
            class: 0,
            bit_gravity: 0,
            win_gravity: 0,
            backing_store: 0,
            backing_planes: 0,
            backing_pixel: 0,
            save_under: 0,
            colormap: 0, // as Colormap,
            map_installed: 0,
            map_state: 0,
            all_event_masks: 0,
            your_event_mask: 0,
            do_not_propagate_mask: 0,
            override_redirect: 0,
            screen: std::ptr::null_mut::<Screen>(),
        }
    }
}

pub type Status = i32;

pub type XPointer = *mut libc::c_char;

pub const ZPixmap: i32 = 2; /* depth == drawable depth */

#[derive(Debug)]
#[repr(C)]
pub struct funcs {
    /* image manipulation routines */
    // Stub these for now, These are not the real signatures.
    pub create_image: unsafe extern "C" fn(i32) -> i32,
    pub destroy_image: unsafe extern "C" fn(i32) -> i32,
    pub get_pixel: unsafe extern "C" fn(i32) -> i32,
    pub put_pixel: unsafe extern "C" fn(i32) -> i32,
    pub sub_image: unsafe extern "C" fn(i32) -> i32,
    pub add_pixel: unsafe extern "C" fn(i32) -> i32,
}
#[derive(Debug)]
#[repr(C)]
pub struct XImage {
    pub width: i32,
    pub height: i32,             /* size of image */
    pub xoffset: i32,            /* number of pixels offset in X direction */
    pub format: i32,             /* XYBitmap, XYPixmap, ZPixmap */
    pub data: *mut libc::c_char, /* pointer to image data */
    pub byte_order: i32,         /* data byte order, LSBFirst, MSBFirst */
    pub bitmap_unit: i32,        /* quant. of scanline 8, 16, 32 */
    pub bitmap_bit_order: i32,   /* LSBFirst, MSBFirst */
    pub bitmap_pad: i32,         /* 8, 16, 32 either XY or ZPixmap */
    pub depth: i32,              /* depth of image */
    pub bytes_per_line: i32,     /* accelarator to next line */
    pub bits_per_pixel: i32,     /* bits per pixel (ZPixmap) */
    pub red_mask: u64,           /* bits in z arrangment */
    pub green_mask: u64,
    pub blue_mask: u64,
    pub obdata: XPointer, /* hook for the object routines to hang on */
    pub f: funcs,
}

pub type ShmSeg = u64;

#[derive(Debug)]
#[repr(C)]
pub struct XShmSegmentInfo {
    pub shmseg: ShmSeg,             /* resource id */
    pub shmid: i32,                 /* kernel id */
    pub shmaddr: *mut libc::c_char, /* address in client */
    pub readOnly: Bool,             /* how the server should attach it */
}
impl Default for XShmSegmentInfo {
    fn default() -> XShmSegmentInfo {
        XShmSegmentInfo {
            shmseg: 0,
            shmid: 0,
            shmaddr: std::ptr::null_mut::<libc::c_char>(),
            readOnly: 0,
        }
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct XErrorEvent {
    type_: i32,
    display: *mut Display, /* Display the event was read from */
    serial: u64,           /* serial number of failed request */
    error_code: u8,        /* error code of failed request */
    request_code: u8,      /* Major op-code of failed request */
    minor_code: u8,        /* Minor op-code of failed request */
    resourceid: XID,       /* resource id */
}

/*
    https://www.remlab.net/op/xlib.shtml
    typedef int (*XErrorHandler)(Display *, XErrorEvent *);
    typedef int (*XIOErrorHandler)(Display *);

    XErrorHandler XSetErrorHandler(XErrorHandler handler);
    XIOErrorHandler XSetIOErrorHandler(XIOErrorHandler handler);
*/

type XErrorHandler = unsafe extern "C" fn(*mut Display, *mut XErrorEvent) -> i32;

pub const AllPlanes: u64 = 0xFFFFFFFFFFFFFFFF;

#[link(name = "X11")]
extern "C" {
    pub fn XOpenDisplay(text: *const libc::c_char) -> *mut Display;

    pub fn XRootWindow(display: *mut Display, screen_number: i32) -> Window;
    pub fn XDefaultScreen(display: *mut Display) -> i32;

    pub fn XGetWindowAttributes(
        display: *mut Display,
        window: Window,
        attributes: *mut XWindowAttributes,
    ) -> Status;

    pub fn XDestroyImage(ximage: *mut XImage) -> i32;

    pub fn XSetErrorHandler(handler: XErrorHandler) -> XErrorHandler;

    pub fn XSync(display: *mut Display, discard: Bool);
    pub fn XFlush(display: *mut Display);

    pub fn XGetGeometry(
        display: *mut Display,
        drawable: Drawable,
        root_return: *mut Window,
        x_return: *mut i32,
        y_return: *mut i32,
        width_return: *mut u32,
        height_return: *mut u32,
        border_width_return: *mut u32,
        depth_return: *mut u32,
    ) -> Status;
}

#[link(name = "Xext")]
extern "C" {
    pub fn XShmQueryExtension(display: *mut Display) -> Bool;

    pub fn XShmCreateImage(
        display: *mut Display,
        visual: *mut Visual,
        depth: u32,
        format: i32,
        data: *mut libc::c_char,
        info: *mut XShmSegmentInfo,
        width: u32,
        height: u32,
    ) -> *mut XImage;

    pub fn XShmAttach(display: *mut Display, shminfo: *const XShmSegmentInfo) -> Bool;
    pub fn XShmGetImage(
        display: *mut Display,
        d: Drawable,
        image: *mut XImage,
        x: i32,
        y: i32,
        plane_mask: u64,
    ) -> bool;

}
