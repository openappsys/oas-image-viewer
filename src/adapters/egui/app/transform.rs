use super::types::{EguiApp, ReadonlyTransformState};
use egui::Context;
use image::{DynamicImage, RgbaImage};

impl EguiApp {
    pub(crate) fn handle_rotate_clockwise(&mut self, ctx: &Context) {
        self.readonly_transform.rotation_quarters =
            (self.readonly_transform.rotation_quarters + 1) % 4;
        self.apply_readonly_transform_texture(ctx);
    }

    pub(crate) fn handle_rotate_counterclockwise(&mut self, ctx: &Context) {
        self.readonly_transform.rotation_quarters =
            (self.readonly_transform.rotation_quarters + 3) % 4;
        self.apply_readonly_transform_texture(ctx);
    }

    pub(crate) fn handle_flip_horizontal(&mut self, ctx: &Context) {
        self.readonly_transform.flip_horizontal = !self.readonly_transform.flip_horizontal;
        self.apply_readonly_transform_texture(ctx);
    }

    pub(crate) fn handle_flip_vertical(&mut self, ctx: &Context) {
        self.readonly_transform.flip_vertical = !self.readonly_transform.flip_vertical;
        self.apply_readonly_transform_texture(ctx);
    }

    pub(crate) fn reset_readonly_transform(&mut self) {
        self.readonly_transform = ReadonlyTransformState::default();
    }

    pub(crate) fn apply_readonly_transform_texture(&mut self, ctx: &Context) {
        let Some((width, height, base_rgba_data)) = self.current_texture_data.clone() else {
            return;
        };
        let Some(base_image) =
            RgbaImage::from_raw(width as u32, height as u32, base_rgba_data)
        else {
            return;
        };

        let transformed = apply_transform(
            DynamicImage::ImageRgba8(base_image),
            self.readonly_transform.rotation_quarters,
            self.readonly_transform.flip_horizontal,
            self.readonly_transform.flip_vertical,
        );
        let rgba = transformed.to_rgba8();
        let new_width = rgba.width() as usize;
        let new_height = rgba.height() as usize;
        let data = rgba.into_raw();
        let image_data = egui::ColorImage::from_rgba_unmultiplied([new_width, new_height], &data);

        let texture_name = if let Some((path, _)) = &self.current_texture {
            format!("{}_readonly_transform", path)
        } else {
            "readonly_transform".to_string()
        };
        let texture =
            ctx.load_texture(texture_name.clone(), image_data, egui::TextureOptions::LINEAR);
        self.current_texture = Some((texture_name, texture));
    }
}

fn apply_transform(
    mut image: DynamicImage,
    rotation_quarters: u8,
    flip_horizontal: bool,
    flip_vertical: bool,
) -> DynamicImage {
    for _ in 0..rotation_quarters % 4 {
        image = image.rotate90();
    }
    if flip_horizontal {
        image = image.fliph();
    }
    if flip_vertical {
        image = image.flipv();
    }
    image
}

#[cfg(test)]
mod tests {
    use super::apply_transform;
    use image::{DynamicImage, Rgba, RgbaImage};

    #[test]
    fn transform_rotate_90_changes_dimensions() {
        let img = DynamicImage::ImageRgba8(RgbaImage::from_pixel(2, 3, Rgba([1, 2, 3, 255])));
        let out = apply_transform(img, 1, false, false);
        assert_eq!(out.width(), 3);
        assert_eq!(out.height(), 2);
    }

    #[test]
    fn transform_flip_keeps_dimensions() {
        let img = DynamicImage::ImageRgba8(RgbaImage::from_pixel(2, 3, Rgba([1, 2, 3, 255])));
        let out = apply_transform(img, 0, true, true);
        assert_eq!(out.width(), 2);
        assert_eq!(out.height(), 3);
    }
}
