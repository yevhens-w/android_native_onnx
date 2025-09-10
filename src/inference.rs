/// Core ONNX inference functionality
use crate::constants::{IMAGE_HEIGHT, IMAGE_WIDTH, IMAGENET_MEAN, IMAGENET_STD, TOP_K_PREDICTIONS, MIN_CLASSIFICATION_CLASSES};
use crate::errors::{InferenceError, InferenceResult};
use crate::labels::LabelsManager;
use crate::types::{ClassificationResult, InferenceResult as InferenceOutput};
use ndarray::Array4;
use ort::{session::Session, value::Value};
use std::sync::Mutex;
use std::time::Instant;

/// Static storage for last inference result
static LAST_RESULT: Mutex<Option<InferenceOutput>> = Mutex::new(None);

/// Static storage for last error message
static LAST_ERROR: Mutex<Option<String>> = Mutex::new(None);

/// Static storage for single cached ONNX session
static CACHED_SESSION: Mutex<Option<(String, Session)>> = Mutex::new(None);

/// ONNX inference engine
pub struct InferenceEngine;

impl InferenceEngine {
    /// Preprocess image bytes into normalized tensor
    fn preprocess_image(image_bytes: &[u8]) -> InferenceResult<Array4<f32>> {
        // Load image from bytes
        let img = image::load_from_memory(image_bytes)
            .map_err(|e| InferenceError::invalid_image(format!("Failed to load image from bytes: {}", e)))?;

        // Resize to required dimensions
        let resized = img.resize_exact(IMAGE_WIDTH, IMAGE_HEIGHT, image::imageops::FilterType::Lanczos3);
        let rgb_img = resized.to_rgb8();

        // Create normalized tensor
        let mut input_array = Array4::<f32>::zeros((1, 3, IMAGE_HEIGHT as usize, IMAGE_WIDTH as usize));

        for (x, y, pixel) in rgb_img.enumerate_pixels() {
            let [r, g, b] = pixel.0;
            
            // Normalize using ImageNet statistics
            input_array[[0, 0, y as usize, x as usize]] = (r as f32 / 255.0 - IMAGENET_MEAN[0]) / IMAGENET_STD[0];
            input_array[[0, 1, y as usize, x as usize]] = (g as f32 / 255.0 - IMAGENET_MEAN[1]) / IMAGENET_STD[1];
            input_array[[0, 2, y as usize, x as usize]] = (b as f32 / 255.0 - IMAGENET_MEAN[2]) / IMAGENET_STD[2];
        }

        Ok(input_array)
    }

    /// Apply softmax activation to raw logits
    fn softmax(input: &[f32]) -> Vec<f32> {
        let max_val = input.iter().fold(f32::NEG_INFINITY, |a, &b| a.max(b));
        let exp_values: Vec<f32> = input.iter().map(|&x| (x - max_val).exp()).collect();
        let sum: f32 = exp_values.iter().sum();
        exp_values.iter().map(|&x| x / sum).collect()
    }

    /// Get top K predictions from probabilities
    fn get_top_predictions(probabilities: &[f32], k: usize) -> Vec<ClassificationResult> {
        let mut indexed_probs: Vec<(usize, f32)> = probabilities
            .iter()
            .enumerate()
            .map(|(i, &prob)| (i, prob))
            .collect();
            
        indexed_probs.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        indexed_probs
            .iter()
            .take(k)
            .map(|&(idx, prob)| {
                ClassificationResult::new(idx, LabelsManager::get_label(idx), prob)
            })
            .collect()
    }

    /// Load ONNX model from file and cache it (replaces any existing cached model)
    pub fn load_model(model_path: &str) -> InferenceResult<()> {
        // Check if model file exists
        if !std::path::Path::new(model_path).exists() {
            return Err(InferenceError::model_not_found(model_path));
        }

        // Check if this model is already cached
        {
            if let Ok(cached_session) = CACHED_SESSION.lock() {
                if let Some((cached_path, _)) = cached_session.as_ref() {
                    if cached_path == model_path {
                        return Ok(()); // Same model already loaded
                    }
                }
            }
        }

        // Read model bytes
        let model_bytes = std::fs::read(model_path)
            .map_err(|e| InferenceError::model_loading_failed(format!("Failed to read model file {}: {}", model_path, e)))?;

        // Create ONNX session
        let session = Session::builder()
            .map_err(|e| InferenceError::session_failed(format!("Failed to create ONNX session builder: {:?}", e)))?
            .commit_from_memory(&model_bytes)
            .map_err(|e| InferenceError::model_loading_failed(format!("Failed to load model from memory: {:?}", e)))?;

        // Cache the session (replacing any existing cached session)
        if let Ok(mut cached_session) = CACHED_SESSION.lock() {
            *cached_session = Some((model_path.to_string(), session));
        } else {
            return Err(InferenceError::memory_error("Failed to acquire session cache mutex"));
        }

        Ok(())
    }

    /// Run inference using the currently cached session
    pub fn run_inference(image_bytes: &[u8]) -> InferenceResult<InferenceOutput> {
        // Preprocess image with timing
        let preprocess_start = Instant::now();
        let input_array = Self::preprocess_image(image_bytes)?;
        let input_data = input_array.into_raw_vec();
        let preprocessing_time_ms = preprocess_start.elapsed().as_secs_f32() * 1000.0;

        let mut cached_session = CACHED_SESSION.lock()
            .map_err(|_| InferenceError::memory_error("Failed to acquire session cache mutex"))?;

        if let Some((_cached_path, session)) = cached_session.as_mut() {
            // Create input tensor
            let input_tensor = Value::from_array(([1, 3, IMAGE_HEIGHT as i64, IMAGE_WIDTH as i64], input_data))
                .map_err(|e| InferenceError::inference_failed(format!("Failed to create input tensor: {:?}", e)))?;

            // Run inference with timing
            let inference_start = Instant::now();
            let input_name = session.inputs[0].name.clone();
            let inputs = ort::inputs![input_name.as_str() => input_tensor];
            let outputs = session
                .run(inputs)
                .map_err(|e| InferenceError::inference_failed(format!("Inference execution failed: {:?}", e)))?;
            let inference_time_ms = inference_start.elapsed().as_secs_f32() * 1000.0;

            // Process output with timing
            let postprocess_start = Instant::now();
            if let Some(output) = outputs.values().next() {
                let shape = output.shape().iter().map(|&x| x as usize).collect::<Vec<_>>();
                let (_output_shape, data_slice) = output
                    .try_extract_tensor::<f32>()
                    .map_err(|e| InferenceError::output_processing_failed(format!("Failed to extract tensor data: {:?}", e)))?;
                let data = data_slice.to_vec();

                // Determine if this is a classification model and compute predictions
                let (is_classification, top_predictions) = if data.len() >= MIN_CLASSIFICATION_CLASSES {
                    let probabilities = Self::softmax(&data);
                    let predictions = Self::get_top_predictions(&probabilities, TOP_K_PREDICTIONS);
                    (true, predictions)
                } else {
                    (false, Vec::new())
                };

                let postprocessing_time_ms = postprocess_start.elapsed().as_secs_f32() * 1000.0;

                let result = InferenceOutput::new_with_timing(
                    data, 
                    shape, 
                    is_classification, 
                    top_predictions,
                    inference_time_ms,
                    preprocessing_time_ms,
                    postprocessing_time_ms
                );

                // Store result for later retrieval (for JNI compatibility)
                if let Ok(mut last_result) = LAST_RESULT.lock() {
                    *last_result = Some(result.clone());
                }

                Ok(result)
            } else {
                Err(InferenceError::output_processing_failed("No output from model"))
            }
        } else {
            Err(InferenceError::model_not_found("No model loaded. Call load_model first."))
        }
    }

    /// Check if any model is currently loaded in cache
    pub fn is_model_loaded() -> bool {
        if let Ok(cached_session) = CACHED_SESSION.lock() {
            cached_session.is_some()
        } else {
            false
        }
    }

    /// Get the path of the currently loaded model
    pub fn get_loaded_model_path() -> Option<String> {
        if let Ok(cached_session) = CACHED_SESSION.lock() {
            cached_session.as_ref().map(|(path, _)| path.clone())
        } else {
            None
        }
    }

    /// Get the last inference result (for JNI compatibility)
    pub fn get_last_result() -> Option<InferenceOutput> {
        LAST_RESULT.lock().ok()?.as_ref().cloned()
    }

    /// Store error message for JNI retrieval
    pub fn store_error(error: &str) {
        if let Ok(mut last_error) = LAST_ERROR.lock() {
            *last_error = Some(error.to_string());
        }
    }

    /// Get last error message (for JNI compatibility)
    pub fn get_last_error() -> Option<String> {
        LAST_ERROR.lock().ok()?.as_ref().cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_softmax() {
        let input = vec![1.0, 2.0, 3.0];
        let output = InferenceEngine::softmax(&input);
        
        // Check sum equals 1.0
        let sum: f32 = output.iter().sum();
        assert!((sum - 1.0).abs() < 1e-6);
        
        // Check monotonicity (larger input -> larger output)
        assert!(output[0] < output[1]);
        assert!(output[1] < output[2]);
    }

    #[test]
    fn test_top_predictions() {
        let probs = vec![0.1, 0.7, 0.2];
        let predictions = InferenceEngine::get_top_predictions(&probs, 2);
        
        assert_eq!(predictions.len(), 2);
        assert_eq!(predictions[0].class_id, 1); // Index of highest prob (0.7)
        assert_eq!(predictions[1].class_id, 2); // Index of second highest (0.2)
    }
}