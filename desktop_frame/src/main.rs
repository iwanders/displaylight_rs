use desktop_frame;
fn main() {
    let mut grabber = desktop_frame::get_grabber();

    let res = grabber.get_resolution();

    println!("Grabber reports resolution of: {:?}", res);
    grabber.prepare_capture(1920, 0, res.width - 1920, res.height);

    let res = grabber.capture_image();
    println!("Grabber tried to capture image, succes? {}", res);
    let img = grabber.get_image();
    println!("Grabber writing to temp.");
    img.write_pnm("/tmp/foo.pnm").unwrap();
    println!("Grabber done writing");

    println!("Cloning image.");

    let z = img.clone();
    println!("Grabber writing to temp.");
    z.write_pnm("/tmp/z.pnm").unwrap();
    println!("Grabber done writing");
    println!("First pixel: {:#?}", img.get_pixel(0, 0));
}
