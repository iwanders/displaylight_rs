use desktop_frame;

use std::env::temp_dir;
fn main() {
    let mut grabber = desktop_frame::get_grabber();

    let res = grabber.get_resolution();

    println!("Grabber reports resolution of: {:?}", res);
    grabber.prepare_capture(1920, 0, res.width - 1920, res.height);

    let res = grabber.capture_image();
    println!("Grabber tried to capture image, succes? {}", res);
    let img = grabber.get_image();
    println!("Grabber writing to temp {:?}", temp_dir());
    img.write_ppm(
        temp_dir()
            .join("foo.ppm")
            .to_str()
            .expect("path must be ok"),
    )
    .unwrap();
    println!("Grabber done writing");

    let z = desktop_frame::read_ppm(
        temp_dir()
            .join("foo.ppm")
            .to_str()
            .expect("path must be ok"),
    )
    .expect("must be good");
    z.write_ppm(
        temp_dir()
            .join("bar.ppm")
            .to_str()
            .expect("path must be ok"),
    )
    .unwrap();

    println!("Cloning image.");

    let z = img.clone();
    println!("Grabber writing to temp.");
    z.write_ppm(temp_dir().join("z.ppm").to_str().expect("path must be ok"))
        .unwrap();
    println!("Grabber done writing");
    println!("First pixel: {:#?}", img.get_pixel(0, 0));
}
