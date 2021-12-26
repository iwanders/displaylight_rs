use desktop;
fn main() {
    let mut grabber = desktop::get_grabber();
    grabber.capture_image();
    println!("Hello, world!");
}
