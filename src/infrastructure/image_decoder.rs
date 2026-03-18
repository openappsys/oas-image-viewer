use crate::core::{CoreError, Result};
use image::DynamicImage;
use std::path::Path;

pub trait ImageDecoderBackend: Send + Sync {
    fn decode_path(&self, path: &Path) -> Result<DynamicImage>;
    fn decode_bytes(&self, data: &[u8]) -> Result<DynamicImage>;
    fn dimensions(&self, path: &Path) -> Result<(u32, u32)>;
}

pub struct StandardImageDecoderBackend;

impl StandardImageDecoderBackend {
    pub fn new() -> Self {
        Self
    }
}

impl Default for StandardImageDecoderBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl ImageDecoderBackend for StandardImageDecoderBackend {
    fn decode_path(&self, path: &Path) -> Result<DynamicImage> {
        match image::open(path) {
            Ok(img) => Ok(img),
            Err(_) => {
                let data = std::fs::read(path).map_err(|e| {
                    CoreError::technical("STORAGE_ERROR", format!("Failed to read file: {}", e))
                })?;
                self.decode_bytes(&data)
            }
        }
    }

    fn decode_bytes(&self, data: &[u8]) -> Result<DynamicImage> {
        image::load_from_memory(data).map_err(|e| {
            CoreError::technical(
                "INVALID_IMAGE_FORMAT",
                format!("Failed to decode image: {}", e),
            )
        })
    }

    fn dimensions(&self, path: &Path) -> Result<(u32, u32)> {
        let reader = image::ImageReader::open(path).map_err(|e| {
            CoreError::technical(
                "STORAGE_ERROR",
                format!("Failed to open image for dimensions: {}", e),
            )
        })?;
        match reader.into_dimensions() {
            Ok(dimensions) => Ok(dimensions),
            Err(_) => {
                let img = self.decode_path(path)?;
                Ok((img.width(), img.height()))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{ImageDecoderBackend, StandardImageDecoderBackend};

    #[test]
    fn decode_invalid_bytes_returns_error() {
        let backend = StandardImageDecoderBackend::new();
        let result = backend.decode_bytes(&[0u8; 16]);
        assert!(result.is_err());
    }
}
