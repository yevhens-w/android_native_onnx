use crate::constants::{TOP_K_PREDICTIONS, MIN_CLASSIFICATION_CLASSES};
use crate::errors::{InferenceError, InferenceResult};
use crate::image_processor::ImageProcessor;
use crate::labels::LabelsManager;
use crate::session_manager::SessionManager;
use crate::tensor_utils::TensorUtils;
use crate::types::{ClassificationResult, InferenceResult as InferenceOutput};
use std::sync::Mutex;
use std::time::Instant;

static LAST_RESULT: Mutex<Option<InferenceOutput>> = Mutex::new(None);
static LAST_ERROR: Mutex<Option<String>> = Mutex::new(None);

pub struct InferenceEngine;

impl InferenceEngine {
    fn softmax(input: &[f32]) -> Vec<f32> {
        let max_val = input.iter().fold(f32::NEG_INFINITY, |a, &b| a.max(b));
        let exp_values: Vec<f32> = input.iter().map(|&x| (x - max_val).exp()).collect();
        let sum: f32 = exp_values.iter().sum();
        exp_values.iter().map(|&x| x / sum).collect()
    }

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

    pub fn load_model(model_path: &str) -> InferenceResult<()> {
        SessionManager::load_model(model_path)
    }

    pub fn run_inference(image_bytes: &[u8]) -> InferenceResult<InferenceOutput> {
        let preprocess_start = Instant::now();
        let input_array = ImageProcessor::preprocess_image(image_bytes)?;
        let input_data = input_array.into_raw_vec();
        let preprocessing_time_ms = preprocess_start.elapsed().as_secs_f32() * 1000.0;

        let current_model_path = SessionManager::get_current_model_path()
            .ok_or_else(|| InferenceError::model_not_found("No model selected. Call set_current_model first."))?;

        let mut cached_sessions = SessionManager::get_session(&current_model_path)?;

        if let Some(session) = cached_sessions.get_mut(&current_model_path) {
            let input_info = &session.inputs[0];
            let expected_type = input_info.input_type.tensor_type().unwrap();
            let input_tensor = TensorUtils::create_input_tensor(input_data, expected_type)?;

            let inference_start = Instant::now();
            let input_name = session.inputs[0].name.clone();
            let inputs = ort::inputs![input_name.as_str() => input_tensor];
            let outputs = session
                .run(inputs)
                .map_err(|e| InferenceError::inference_failed(format!("Inference execution failed: {:?}", e)))?;
            let inference_time_ms = inference_start.elapsed().as_secs_f32() * 1000.0;

            let postprocess_start = Instant::now();
            if let Some(output) = outputs.values().next() {
                let shape = TensorUtils::get_tensor_shape(&output);
                let data = TensorUtils::extract_output_data(&output)?;

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

                if let Ok(mut last_result) = LAST_RESULT.lock() {
                    *last_result = Some(result.clone());
                }

                Ok(result)
            } else {
                Err(InferenceError::output_processing_failed("No output from model"))
            }
        } else {
            Err(InferenceError::model_not_found(&format!("Model not found in cache: {}. Call load_model first.", current_model_path)))
        }
    }

    pub fn is_model_loaded() -> bool {
        SessionManager::is_model_loaded()
    }

    pub fn get_loaded_model_path() -> Option<String> {
        SessionManager::get_current_model_path()
    }

    pub fn set_current_model(model_path: &str) -> InferenceResult<()> {
        SessionManager::set_current_model(model_path)
    }

    pub fn get_last_result() -> Option<InferenceOutput> {
        LAST_RESULT.lock().ok()?.as_ref().cloned()
    }

    pub fn store_error(error: &str) {
        if let Ok(mut last_error) = LAST_ERROR.lock() {
            *last_error = Some(error.to_string());
        }
    }

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