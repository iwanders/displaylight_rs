use crate::interface::*;
use windows;

// This uses the desktop duplication api.
// https://docs.microsoft.com/en-us/windows/win32/direct3ddxgi/desktop-dup-api
use windows::{
    core::Result, Win32::Graphics::Direct3D::*, Win32::Graphics::Direct3D12::*,
    Win32::Graphics::Dxgi::Common::*, Win32::Graphics::Dxgi::*,
};

struct ImageWin {}

impl ImageWin {}

impl Image for ImageWin {
    fn get_width(&self) -> u32 {
        0
    }
    fn get_height(&self) -> u32 {
        0
    }
    fn get_pixel(&self, x: u32, y: u32) -> RGB {
        RGB { r: 0, g: 0, b: 0 }
    }
}

/*
From the C++ project.
  initAdapter();
  initOutput();
  initDevice();
  initDuplicator();

*/

// from  https://github.com/microsoft/windows-samples-rs/blob/5d67b33e7115ec1dd4f8448301bf6ce794c93b5f/direct3d12/src/main.rs#L204-L234.
fn get_hardware_adapter(factory: &IDXGIFactory4) -> Result<IDXGIAdapter1> {
    for i in 0.. {
        let adapter = unsafe { factory.EnumAdapters1(i)? };

        let desc = unsafe { adapter.GetDesc1()? };

        if (DXGI_ADAPTER_FLAG::from(desc.Flags) & DXGI_ADAPTER_FLAG_SOFTWARE)
            != DXGI_ADAPTER_FLAG_NONE
        {
            // Don't select the Basic Render Driver adapter. If you want a
            // software adapter, pass in "/warp" on the command line.
            continue;
        }

        // Check to see whether the adapter supports Direct3D 12, but don't
        // create the actual device yet.
        if unsafe {
            D3D12CreateDevice(
                &adapter,
                D3D_FEATURE_LEVEL_11_0,
                std::ptr::null_mut::<Option<ID3D12Device>>(),
            )
        }
        .is_ok()
        {
            return Ok(adapter);
        }
    }

    unreachable!()
}

// from https://github.com/microsoft/windows-samples-rs/blob/5d67b33e7115ec1dd4f8448301bf6ce794c93b5f/direct3d12/src/main.rs#L537
fn create_device() -> Result<(IDXGIFactory4, ID3D12Device)> {
    if cfg!(debug_assertions) {
        unsafe {
            let mut debug: Option<ID3D12Debug> = None;
            if let Some(debug) = D3D12GetDebugInterface(&mut debug).ok().and_then(|_| debug) {
                debug.EnableDebugLayer();
            }
        }
    }

    let dxgi_factory_flags = if cfg!(debug_assertions) {
        DXGI_CREATE_FACTORY_DEBUG
    } else {
        0
    };

    let dxgi_factory: IDXGIFactory4 = unsafe { CreateDXGIFactory2(dxgi_factory_flags) }?;

    let adapter = get_hardware_adapter(&dxgi_factory)?;

    let mut device: Option<ID3D12Device> = None;
    unsafe { D3D12CreateDevice(adapter, D3D_FEATURE_LEVEL_11_0, &mut device) }?;
    Ok((dxgi_factory, device.unwrap()))
}

#[derive(Default)]
struct GrabberWin {
    adaptor: Option<IDXGIAdapter1>,
    output: Option<IDXGIOutput>,
}

impl Drop for GrabberWin {
    fn drop(&mut self) {}
}

use std::ffi::OsString;
use std::os::windows::prelude::*;

impl GrabberWin {
    fn init_adaptor(&mut self) {
        let (factory, device) = create_device().expect("Must have a device.");
        let adaptor = get_hardware_adapter(&factory).expect("Must have an adaptor.");
        self.adaptor = Some(adaptor);
    }

    fn init_output(&mut self, desired: u32) {
        // Now, we break from the example.
        let adaptor = self.adaptor.as_ref().expect("Must have an adaptor.");
        let mut output_index: u32 = 0;
        unsafe {
            let mut res = adaptor.EnumOutputs(output_index);
            while res.is_ok() {
                println!("idxgiouptut:");
                let output = res.unwrap();
                let desc = output.GetDesc().expect("Should hav ea description");
                println!(
                    "Output: {}, name: {}, monitor: {}",
                    output_index,
                    OsString::from_wide(&desc.DeviceName)
                        .to_str()
                        .unwrap_or("Unknown"),
                    desc.Monitor
                );
                if desired == output_index {
                    self.output = Some(output);
                }
                output_index = output_index + 1;
                res = adaptor.EnumOutputs(output_index);
            }
        }
    }

    pub fn new() -> GrabberWin {
        let mut n: GrabberWin = Default::default();
        n.init_adaptor();
        n.init_output(0);
        n
    }
    pub fn prepare(&mut self, x: u32, y: u32, width: u32, height: u32) -> bool {
        true
    }
}

impl Grabber for GrabberWin {
    fn capture_image(&mut self) -> bool {
        false
    }
    fn get_image(&mut self) -> Box<dyn Image> {
        Box::<ImageWin>::new(ImageWin {})
    }

    fn get_resolution(&mut self) -> Resolution {
        Resolution {
            width: 0,
            height: 0,
        }
    }

    fn prepare_capture(&mut self, x: u32, y: u32, width: u32, height: u32) -> bool {
        return GrabberWin::prepare(self, x, y, width, height);
    }
}

pub fn get_grabber() -> Box<dyn Grabber> {
    let mut z = Box::<GrabberWin>::new(GrabberWin::new());
    z.prepare(0, 0, 0, 0);
    z
}
