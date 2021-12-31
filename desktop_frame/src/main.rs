use desktop_frame;

use std::env::temp_dir;
fn main() {
    let mut grabber = desktop_frame::get_grabber();

    let res = grabber.get_resolution();

    println!("Grabber reports resolution of: {:?}", res);
    grabber.prepare_capture(0, 1920, 0, res.width - 1920, res.height);

    let mut res = grabber.capture_image();
    while !res {
        res = grabber.capture_image();
    }

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
    z.write_bmp(temp_dir().join("z.bmp").to_str().expect("path must be ok"))
        .unwrap();
    println!("Grabber done writing");
    println!("First pixel: {:#?}", img.get_pixel(0, 0));
    println!(
        "last pixel: {:#?}",
        img.get_pixel(img.get_width() - 1, img.get_height() - 1)
    );

    for _i in 0..2 {
        let res = grabber.capture_image();
        println!("Grabber tried to capture image, succes? {}", res);
        let img = grabber.get_image();
        println!(
            "last pixel: {:#?}",
            img.get_pixel(img.get_width() - 1, img.get_height() - 1)
        );
    }
}
