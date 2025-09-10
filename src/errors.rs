/// Error handling for ONNX inference operations
use std::fmt;

/// Custom error type for inference operations
#[derive(Debug, Clone)]
pub enum InferenceError {
    /// Model file not found or inaccessible
    ModelNotFound(String),
    /// Invalid image data or format
    InvalidImageData(String),
    /// ONNX Runtime session creation failed
    SessionCreationFailed(String),
    /// Model loading failed
    ModelLoadingFailed(String),
    /// Inference execution failed
    InferenceFailed(String),
    /// Output processing failed
    OutputProcessingFailed(String),
    /// Labels loading failed
    LabelsLoadingFailed(String),
    /// Memory allocation failed
    MemoryError(String),
}

impl fmt::Display for InferenceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InferenceError::ModelNotFound(path) => write!(f, "Model file not found: {}", path),
            InferenceError::InvalidImageData(msg) => write!(f, "Invalid image data: {}", msg),
            InferenceError::SessionCreationFailed(msg) => write!(f, "Failed to create ONNX session: {}", msg),
            InferenceError::ModelLoadingFailed(msg) => write!(f, "Failed to load model: {}", msg),
            InferenceError::InferenceFailed(msg) => write!(f, "Inference execution failed: {}", msg),
            InferenceError::OutputProcessingFailed(msg) => write!(f, "Failed to process output: {}", msg),
            InferenceError::LabelsLoadingFailed(msg) => write!(f, "Failed to load labels: {}", msg),
            InferenceError::MemoryError(msg) => write!(f, "Memory allocation failed: {}", msg),
        }
    }
}

impl std::error::Error for InferenceError {}

/// Result type alias for inference operations
pub type InferenceResult<T> = Result<T, InferenceError>;

/// Utility functions for error conversion
impl InferenceError {
    /// Create a model not found error
    pub fn model_not_found<S: Into<String>>(path: S) -> Self {
        InferenceError::ModelNotFound(path.into())
    }

    /// Create an invalid image data error
    pub fn invalid_image<S: Into<String>>(msg: S) -> Self {
        InferenceError::InvalidImageData(msg.into())
    }

    /// Create a session creation error
    pub fn session_failed<S: Into<String>>(msg: S) -> Self {
        InferenceError::SessionCreationFailed(msg.into())
    }

    /// Create a model loading error
    pub fn model_loading_failed<S: Into<String>>(msg: S) -> Self {
        InferenceError::ModelLoadingFailed(msg.into())
    }

    /// Create an inference execution error
    pub fn inference_failed<S: Into<String>>(msg: S) -> Self {
        InferenceError::InferenceFailed(msg.into())
    }

    /// Create an output processing error
    pub fn output_processing_failed<S: Into<String>>(msg: S) -> Self {
        InferenceError::OutputProcessingFailed(msg.into())
    }

    /// Create a labels loading error
    pub fn labels_loading_failed<S: Into<String>>(msg: S) -> Self {
        InferenceError::LabelsLoadingFailed(msg.into())
    }

    /// Create a memory error
    pub fn memory_error<S: Into<String>>(msg: S) -> Self {
        InferenceError::MemoryError(msg.into())
    }
}

/// Convert from various error types
impl From<image::ImageError> for InferenceError {
    fn from(err: image::ImageError) -> Self {
        InferenceError::InvalidImageData(err.to_string())
    }
}

impl From<ort::Error> for InferenceError {
    fn from(err: ort::Error) -> Self {
        InferenceError::InferenceFailed(format!("ONNX Runtime error: {:?}", err))
    }
}

impl From<std::io::Error> for InferenceError {
    fn from(err: std::io::Error) -> Self {
        InferenceError::ModelNotFound(err.to_string())
    }
}