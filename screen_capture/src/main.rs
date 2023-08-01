use std::env::temp_dir;

fn create_gradient() {
    use screen_capture::Image;
    let mut img = screen_capture::raster_image::RasterImage::filled(1920, 1080, screen_capture::RGB { r: 0, g: 0, b: 0 });
    img.set_gradient(0, 1920, 0, 1080);
    img.write_bmp(
        temp_dir()
            .join("gradient.bmp")
            .to_str()
            .expect("path must be ok"),
    )
    .unwrap();
}

fn main() {
    if false {
        create_gradient();
    }

    let mut grabber = screen_capture::get_capture();

    let res = grabber.get_resolution();

    println!("Capture reports resolution of: {:?}", res);
    grabber.prepare_capture(0, 1920, 0, res.width - 1920, res.height);

    let mut res = grabber.capture_image();
    while !res {
        res = grabber.capture_image();
    }

    println!("Capture tried to capture image, succes? {}", res);
    let img = grabber.get_image();
    println!("Capture writing to temp {:?}", temp_dir());
    img.write_ppm(
        temp_dir()
            .join("foo.ppm")
            .to_str()
            .expect("path must be ok"),
    )
    .unwrap();
    println!("Capture done writing");

    let z = screen_capture::read_ppm(
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
    println!("Capture writing to temp.");
    z.write_ppm(temp_dir().join("z.ppm").to_str().expect("path must be ok"))
        .unwrap();
    z.write_bmp(temp_dir().join("z.bmp").to_str().expect("path must be ok"))
        .unwrap();
    println!("Capture done writing");
    println!("First pixel: {:#?}", img.get_pixel(0, 0));
    println!(
        "last pixel: {:#?}",
        img.get_pixel(img.get_width() - 1, img.get_height() - 1)
    );

    for _i in 0..2 {
        let res = grabber.capture_image();
        println!("Capture tried to capture image, succes? {}", res);
        let img = grabber.get_image();
        println!(
            "last pixel: {:#?}",
            img.get_pixel(img.get_width() - 1, img.get_height() - 1)
        );
    }
}
