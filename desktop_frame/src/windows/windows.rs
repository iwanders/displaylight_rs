use crate::interface::*;
use windows;

// This uses the desktop duplication api.
// https://docs.microsoft.com/en-us/windows/win32/direct3ddxgi/desktop-dup-api
use windows::{
    core::Result, Win32::Graphics::Direct3D::*, Win32::Graphics::Direct3D11::*,
    Win32::Graphics::Dxgi::Common::*, Win32::Graphics::Dxgi::*,
};

use windows::{
    core::*, Win32::Foundation::*, Win32::Foundation::*, Win32::Graphics::Direct3D::Fxc::*,
    Win32::Graphics::Direct3D::*, Win32::Graphics::Direct3D11::*, Win32::Graphics::Dxgi::Common::*,
    Win32::Graphics::Dxgi::*, Win32::System::Com::*, Win32::System::LibraryLoader::*,
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

// For d3d12 we could follow  https://github.com/microsoft/windows-samples-rs/blob/5d67b33e7115ec1dd4f8448301bf6ce794c93b5f/direct3d12/src/main.rs#L204-L234.

#[derive(Default)]
struct GrabberWin {
    adaptor: Option<IDXGIAdapter1>,
    device: Option<ID3D11Device>,
    device_context: Option<ID3D11DeviceContext>,
    output: Option<IDXGIOutput>,
    duplicator: Option<IDXGIOutputDuplication>,
}

impl Drop for GrabberWin {
    fn drop(&mut self) {}
}

use std::ffi::OsString;
use std::os::windows::prelude::*;

// Apparently from_wide from OsString doesn't respect zero termination.
fn from_wide(arr: &[u16]) -> OsString {
    let len = arr.iter().take_while(|c| **c != 0).count();
    OsString::from_wide(&arr[..len])
}

impl GrabberWin {
    fn init_adaptor(&mut self) -> Result<()> {
        // let (factory, device) = create_device().expect("Must have a device.");
        // let adaptor = get_hardware_adapter(&factory).expect("Must have an adaptor.");
        // self.adaptor = Some(adaptor);

        let dxgi_factory_flags = DXGI_CREATE_FACTORY_DEBUG;
        let factory: IDXGIFactory4 = unsafe { CreateDXGIFactory2(dxgi_factory_flags) }?;

        for i in 0.. {
            let adapter = unsafe { factory.EnumAdapters1(i)? };

            let desc = unsafe { adapter.GetDesc1()? };

            // Skip the software adaptor.
            if (DXGI_ADAPTER_FLAG::from(desc.Flags) & DXGI_ADAPTER_FLAG_SOFTWARE)
                != DXGI_ADAPTER_FLAG_NONE
            {
                continue;
            }

            // Print some info about the adapter.
            println!(
                "Adapter {} -> {:#?} with {} memory",
                i,
                from_wide(&desc.Description),
                desc.DedicatedVideoMemory
            );

            // Instantiate the d3d11 device now.
            let sdk_version = windows::Win32::Graphics::Direct3D11::D3D11_SDK_VERSION;
            let create_flags = windows::Win32::Graphics::Direct3D11::D3D11_CREATE_DEVICE_BGRA_SUPPORT
                               | windows::Win32::Graphics::Direct3D11::D3D11_CREATE_DEVICE_DEBUG;
            let mut level_used = windows::Win32::Graphics::Direct3D::D3D_FEATURE_LEVEL_9_3;
            let featureLevels = [
                windows::Win32::Graphics::Direct3D::D3D_FEATURE_LEVEL_11_0,
                windows::Win32::Graphics::Direct3D::D3D_FEATURE_LEVEL_10_1,
                windows::Win32::Graphics::Direct3D::D3D_FEATURE_LEVEL_10_0,
                windows::Win32::Graphics::Direct3D::D3D_FEATURE_LEVEL_9_3,
            ];

            if unsafe {
                D3D11CreateDevice(
                    &adapter,// padapter: Param0, 
                    windows::Win32::Graphics::Direct3D::D3D_DRIVER_TYPE_UNKNOWN, // drivertype: D3D_DRIVER_TYPE, 
                    0, // software: Param2, 
                    create_flags, // flags: D3D11_CREATE_DEVICE_FLAG, 
                    &featureLevels as *const windows::Win32::Graphics::Direct3D::D3D_FEATURE_LEVEL, // pfeaturelevels: *const D3D_FEATURE_LEVEL, 
                    featureLevels.len() as u32, // featurelevels: u32, 
                    sdk_version, // sdkversion: u32, 
                    &mut self.device, // ppdevice: *mut Option<ID3D11Device>, 
                    &mut level_used,// pfeaturelevel: *mut D3D_FEATURE_LEVEL, 
                    &mut self.device_context// ppimmediatecontext: *mut Option<ID3D11DeviceContext>
                )
            }.is_ok()
            {
                self.adaptor = Some(adapter);
                return Ok(()); // we had success.
            };
        }

        Err(windows::core::Error::OK)  // Just to make an error without failure information.
    }

    fn init_output(&mut self, desired: u32) -> Result<()>
    {
        // Obtain the video outputs used by this adaptor.
        // Is the primary screen always the zeroth index??
        let adaptor = self.adaptor.as_ref().expect("Must be called with an adaptor");
        let mut output_index: u32 = 0;
        unsafe {
            let mut res = adaptor.EnumOutputs(output_index);
            while res.is_ok() {
                println!("idxgiouptut:");
                let output = res.unwrap();
                let desc = output.GetDesc()?;
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
                    return Ok(())
                }
                output_index = output_index + 1;
                res = adaptor.EnumOutputs(output_index);
            }
        }
        Err(windows::core::Error::OK)  // Just to make an error without failure information.
    }



    fn init_duplicator(&mut self) -> Result<()> {
        let output = self.output.as_ref().expect("Must have an output");
        self.duplicator = None;
        // let output1: &IDXGIOutput1  = output.cast().expect("Should be castable.");
        // No idea if this is the way...
        unsafe{
            // let output1: &IDXGIOutput1 = std::mem::transmute::<&IDXGIOutput, &IDXGIOutput1>(output);
            let desc = output.GetDesc()?;
            println!("Device: {:?}, monitor: {}", from_wide(&desc.DeviceName), desc.Monitor);
            // let z = output.CheckInterfaceSupport(&IDXGIOutput1::IID);  // Oh.
            // println!("z: {:?}", z);
            let output1: Result<IDXGIOutput1> = output.cast();
            let output1 = output1.expect("SHould have succeeded.");
            // let output1 = output.GetParent::<&IDXGIOutput1>().expect("Yes");
            // From C++, the following can fail with:
            //  E_ACCESSDENIED, when on fullscreen uac prompt
            //  DXGI_ERROR_SESSION_DISCONNECTED, somehow.
            self.duplicator = Some(output1.DuplicateOutput(self.device.as_ref().expect("Must have a device"))?);

            let duplicator = self.duplicator.as_ref().expect("Must have a duplicator now");
            let mut desc: DXGI_OUTDUPL_DESC = DXGI_OUTDUPL_DESC{ModeDesc:DXGI_MODE_DESC{Width: 0, Height: 0, RefreshRate: DXGI_RATIONAL {Numerator: 0, Denominator: 0}, Format: 0, ScanlineOrdering: 0, Scaling: 0}, Rotation: 0, DesktopImageInSystemMemory: windows::Win32::Foundation::BOOL(0)};
            duplicator.GetDesc(&mut desc);
            println!("Duplicator; {}x{} @ {}/{}, in memory: {}", desc.ModeDesc.Width, desc.ModeDesc.Height, desc.ModeDesc.RefreshRate.Numerator, desc.ModeDesc.RefreshRate.Denominator, desc.DesktopImageInSystemMemory.0);

        }
        Ok(())
        // self.duplicator_output = None;

        // output.DuplicateOutput(self.device.as_ref().unwrap("Must have a device",  &mut self.duplicator)).expect("Should be able to make duplicator");
        // let output1: &IDXGIOutput1 =  output.into();
        /*
          // need to convert output to output1.
          IDXGIOutput1* output1;

          HRESULT hr;

          hr = adapter_output_->QueryInterface(__uuidof(IDXGIOutput1), (void**)&output1);
          if (FAILED(hr))
            throw std::runtime_error("Failed to query IDXGIOutput1");

          // If lost access, we must release the previous duplicator and make a new one.
          duplicator_.reset();
          duplicator_output_.reset();

          IDXGIOutputDuplication* z;
          hr = output1->DuplicateOutput(device_.get(), &z);

          if (E_ACCESSDENIED == hr)
          {
            // full screen security prompt.
            return false;
          }
          if (hr == DXGI_ERROR_SESSION_DISCONNECTED)
          {
            // seems bad?
            return false;
          }

          if (FAILED(hr))
          {
            throw std::runtime_error("Failed to duplicate output");
          }

          duplicator_ = releasing(z);
          duplicator_output_ = releasing(output1);

          // DXGI_OUTDUPL_DESC out_desc;
          // duplicator_->GetDesc(&out_desc);
          // If the data was already in memory we could map it directly... but it is not.
          // std::cout << "Already in mem: " << out_desc.DesktopImageInSystemMemory << " " << std::endl;
        */
    }

    pub fn new() -> GrabberWin {
        /*
        From the C++ project.
          initAdapter();
          initOutput();
          initDevice();
          initDuplicator();
        */
        let mut n: GrabberWin = Default::default();
        n.init_adaptor().expect("Should have an adaptor and d3d11 device now.");
        n.init_output(0).expect("Should be able to get the output.");
        n.init_duplicator().expect("Should be able to get the duplicator.");
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
