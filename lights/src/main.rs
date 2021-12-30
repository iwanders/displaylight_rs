use lights;
use lights::RGB;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    use std::{thread, time};

    println!("Ports: {:#?}", lights::available_ports()?);

    let mut control = lights::Lights::new("/dev/ttyACM0")?;

    let mut config = lights::Config::default();
    config.decay_amount = 1;
    control.set_config(&config)?;

    for _ in 0..100 {
        control.fill(255, 255, 255)?;
        thread::sleep(time::Duration::from_millis(100));
    }

    const max_leds: usize = 228;
    let loops = 10;
    for i in 0..(max_leds * loops) {
        let mut leds = [RGB::default(); max_leds];
        leds[i % max_leds] = RGB {
            r: 255,
            g: 255,
            b: 255,
        };
        control.set_leds(&leds)?;
        thread::sleep(time::Duration::from_millis(10));
    }
    Ok(())
}
