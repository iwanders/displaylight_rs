pub mod border_detection;
pub mod rectangle;
pub mod sampler;
pub mod zones;

#[cfg(test)]
mod tests {
    use super::*;
    use desktop_frame::raster_image::make_dummy_gradient;
    use desktop_frame::raster_image::RasterImage;
    use desktop_frame::{Image, RGB};
    use std::env::temp_dir;

    fn tmp_file(name: &str) -> String {
        temp_dir()
            .join(name)
            .to_str()
            .expect("path must be ok")
            .to_owned()
    }

    #[test]
    fn test_full() {
        // Make a dummy image.
        let img = make_dummy_gradient();
        img.write_bmp(
            temp_dir()
                .join("gradient.bmp")
                .to_str()
                .expect("path must be ok"),
        )
        .unwrap();

        // Detect the black borders
        let mut tracked = desktop_frame::tracked_image::TrackedImage::new(Box::new(img));
        let b = border_detection::find_borders(&tracked, 5);
        let mut track_results = tracked.draw_access(0.5);
        track_results.set_pixel(b.x_min, b.y_min, RGB::cyan());
        track_results.set_pixel(b.x_max, b.y_max, RGB::white());
        track_results
            .write_ppm(&tmp_file("test_full_borders.ppm"))
            .expect("Should succeed.");

        // With the edges known, we can make the zones.
        let zones = zones::Zones::make_zones(&b, 100, 100);
        assert_eq!(zones.len(), 228);

        // With the zones known, we can create the sampler.
        let sampler = sampler::Sampler::make_sampler(&zones, 10);

        // With the sampler, we can now sample and get color values.
        tracked.clear_events();
        let values = sampler.sample(&tracked);
        assert_eq!(values.len(), 228);
        let mut track_results = tracked.draw_access(0.5);

        track_results
            .write_ppm(&tmp_file("test_full_sampling.ppm"))
            .expect("Should succeed.");

        // With the values known, we can color the zones appropriately.
        let mut canvas = RasterImage::filled(
            tracked.get_width(),
            tracked.get_height(),
            Default::default(),
        );
        for (i, zone) in zones.iter().enumerate() {
            canvas.fill_rectangle(zone.x_min, zone.x_max, zone.y_min, zone.y_max, values[i])
        }
        canvas
            .write_bmp(
                temp_dir()
                    .join("analysed_canvas.bmp")
                    .to_str()
                    .expect("path must be ok"),
            )
            .unwrap();
    }
}
