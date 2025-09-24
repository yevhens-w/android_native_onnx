use crate::constants::{IMAGE_HEIGHT, IMAGE_WIDTH, IMAGENET_MEAN, IMAGENET_STD};
use crate::errors::{InferenceError, InferenceResult};
use ndarray::Array4;

pub struct ImageProcessor;

impl ImageProcessor {
    pub fn preprocess_image(image_bytes: &[u8]) -> InferenceResult<Array4<f32>> {
        let img = image::load_from_memory(image_bytes)
            .map_err(|e| InferenceError::invalid_image(format!("Failed to load image from bytes: {}", e)))?;

        let resized = img.resize_exact(IMAGE_WIDTH, IMAGE_HEIGHT, image::imageops::FilterType::Lanczos3);
        let rgb_img = resized.to_rgb8();

        let mut input_array = Array4::<f32>::zeros((1, 3, IMAGE_HEIGHT as usize, IMAGE_WIDTH as usize));

        for (x, y, pixel) in rgb_img.enumerate_pixels() {
            let [r, g, b] = pixel.0;

            input_array[[0, 0, y as usize, x as usize]] = (r as f32 / 255.0 - IMAGENET_MEAN[0]) / IMAGENET_STD[0];
            input_array[[0, 1, y as usize, x as usize]] = (g as f32 / 255.0 - IMAGENET_MEAN[1]) / IMAGENET_STD[1];
            input_array[[0, 2, y as usize, x as usize]] = (b as f32 / 255.0 - IMAGENET_MEAN[2]) / IMAGENET_STD[2];
        }

        Ok(input_array)
    }

    #[allow(dead_code)]
    pub fn normalize_pixel_channel(pixel_value: u8, channel_idx: usize) -> f32 {
        (pixel_value as f32 / 255.0 - IMAGENET_MEAN[channel_idx]) / IMAGENET_STD[channel_idx]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_pixel_channel() {
        let normalized = ImageProcessor::normalize_pixel_channel(128, 0);
        let expected = (128.0 / 255.0 - IMAGENET_MEAN[0]) / IMAGENET_STD[0];
        assert!((normalized - expected).abs() < 1e-6);
    }
}