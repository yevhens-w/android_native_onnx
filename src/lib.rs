mod constants;
mod errors;
mod image_processor;
mod inference;
mod jni;
mod labels;
mod panic_handler;
mod session_manager;
mod system_info;
mod tensor_utils;
mod types;

use crate::inference::InferenceEngine;
use crate::types::InferenceResult;
use crate::errors::InferenceError;

pub fn run_inference_internal(
    image_bytes: &[u8],
) -> Result<InferenceResult, InferenceError> {
    match InferenceEngine::run_inference(image_bytes) {
        Ok(result) => Ok(result),
        Err(e) => {
            InferenceEngine::store_error(&e.to_string());
            Err(e)
        }
    }
}
