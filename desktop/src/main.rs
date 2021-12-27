use desktop;
fn main() {
    let mut grabber = desktop::get_grabber();
    let res = grabber.capture_image();
    println!("Hello, world! {}", res);
    let img = grabber.get_image();
    img.write_pnm("/tmp/foo.pnm").unwrap();
}
