use displaylight::{zones, sampler, border_detection};
use lights;

use std::{thread, time};
use std::error::Error;
fn main() -> Result<(), Box<dyn Error>> {

    let mut grabber = desktop_frame::get_grabber();

    let resolution = grabber.get_resolution();

    println!("Grabber reports resolution of: {:?}", resolution);
    grabber.prepare_capture(1920, 0, resolution.width - 1920, resolution.height);

    let mut control = lights::Lights::new("/dev/ttyACM0")?;

    const MAX_LEDS: usize = 228;

    loop
    {
        let res = grabber.capture_image();
        if (!res)
        {
            continue;
        }
        // Then, grab the image.
        let img = grabber.get_image();

        // Detect the black borders
        let borders = border_detection::find_borders(&*img, 5);

        // With the edges known, we can make the zones.
        let zones = zones::Zones::make_zones(&borders, 200, 200);
        assert_eq!(zones.len(), MAX_LEDS);

        // With the zones known, we can create the sampler.
        let sampler = sampler::Sampler::make_sampler(&zones);

        // With the sampler, we can now sample and get color values.
        let values = sampler.sample(&*img);
        assert_eq!(values.len(), MAX_LEDS);

        // Finally, create the lights::RGB array.
        let mut leds = [lights::RGB::default(); MAX_LEDS];
        for i in 0..MAX_LEDS
        {
            leds[i].r = values[i].r;
            leds[i].g = values[i].g;
            leds[i].b = values[i].b;
        }
        control.set_leds(&leds)?;
        thread::sleep(time::Duration::from_millis(10));
    }

    Ok(())



}
