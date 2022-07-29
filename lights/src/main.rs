use lights::RGB;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    use std::{thread, time};

    let port = std::env::args().nth(1);
    if port.is_none() {
        println!("Ports: {:#?}", lights::available_ports()?);
        return Ok(());
    }
    let port = port.unwrap();

    let mut control = lights::Lights::new(&port)?;
    control.set_limit_factor(0.1);
    control.fill(0, 0, 0)?;

    let config = lights::Config {
        decay_amount: 1,
        ..Default::default()
    };
    control.set_config(&config)?;

    // for _ in 0..100 {
        // control.fill(255, 255, 255)?;
        // thread::sleep(time::Duration::from_millis(100));
    // }

    const MAX_LEDS: usize = 10;
    let loops = 10;
    for i in 0..(MAX_LEDS * loops) {
        let mut leds = [RGB::default(); MAX_LEDS];
        let index = i % (MAX_LEDS - 1);
        leds[index] = RGB {
            r: 255,
            g: 255,
            b: 255,
        };
        control.set_leds(&leds)?;
        thread::sleep(time::Duration::from_millis(10));
    }
    // control.fill(0, 0, 0)?;
    Ok(())
}
