mod border_detection;
mod rectangle;
mod sampler;
mod zones;

#[cfg(test)]
mod tests {
    use super::*;
    use desktop_frame::raster_image::make_dummy_gradient;
    use desktop_frame::raster_image::RasterImage;
    use desktop_frame::Image;
    use std::env::temp_dir;

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
        let borders = border_detection::find_borders(&img, 5);

        // With the edges known, we can make the zones.
        let zones = zones::Zones::make_zones(&borders, 100, 100);
        assert_eq!(zones.len(), 228);

        // With the zones known, we can create the sampler.
        let sampler = sampler::Sampler::make_sampler(&zones);

        // With the sampler, we can now sample and get color values.
        let values = sampler.sample(&img);
        assert_eq!(values.len(), 228);

        // With the values known, we can color the zones appropriately.
        let mut canvas = RasterImage::filled(img.get_width(), img.get_height(), Default::default());
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
