use lights::RGB;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    use std::{thread, time};

    println!("Ports: {:#?}", lights::available_ports()?);

    let mut control = lights::Lights::new("/dev/ttyACM0")?;

    let config = lights::Config {
        decay_amount: 1,
        ..Default::default()
    };
    control.set_config(&config)?;

    for _ in 0..100 {
        control.fill(255, 255, 255)?;
        thread::sleep(time::Duration::from_millis(100));
    }

    const MAX_LEDS: usize = 228;
    let loops = 10;
    for i in 0..(MAX_LEDS * loops) {
        let mut leds = [RGB::default(); MAX_LEDS];
        leds[i % MAX_LEDS] = RGB {
            r: 255,
            g: 255,
            b: 255,
        };
        control.set_leds(&leds)?;
        thread::sleep(time::Duration::from_millis(10));
    }
    Ok(())
}
