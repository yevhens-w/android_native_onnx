/// Data structures for ONNX inference results and classification
use std::fmt;

/// Represents a single classification result with class information and confidence
#[derive(Debug, Clone)]
pub struct ClassificationResult {
    pub class_id: usize,
    pub class_name: String,
    pub confidence: f32,
}

impl ClassificationResult {
    /// Create a new classification result
    pub fn new(class_id: usize, class_name: String, confidence: f32) -> Self {
        Self {
            class_id,
            class_name,
            confidence,
        }
    }
}

impl fmt::Display for ClassificationResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Class {} ({}): {:.2}%",
            self.class_id,
            self.class_name,
            self.confidence * 100.0
        )
    }
}

/// Complete inference result containing raw output data and predictions
#[derive(Debug, Clone)]
pub struct InferenceResult {
    pub data: Vec<f32>,
    pub shape: Vec<usize>,
    pub is_classification: bool,
    pub top_predictions: Vec<ClassificationResult>,
    pub inference_time_ms: f32,
    pub preprocessing_time_ms: f32,
    pub postprocessing_time_ms: f32,
    pub total_time_ms: f32,
}

impl InferenceResult {
    /// Create a new inference result
    pub fn new(
        data: Vec<f32>,
        shape: Vec<usize>,
        is_classification: bool,
        top_predictions: Vec<ClassificationResult>,
        inference_time_ms: f32,
        preprocessing_time_ms: f32,
        postprocessing_time_ms: f32,
        total_time_ms: f32,
    ) -> Self {
        Self {
            data,
            shape,
            is_classification,
            top_predictions,
            inference_time_ms,
            preprocessing_time_ms,
            postprocessing_time_ms,
            total_time_ms,
        }
    }

    /// Create a new inference result with timing calculations
    pub fn new_with_timing(
        data: Vec<f32>,
        shape: Vec<usize>,
        is_classification: bool,
        top_predictions: Vec<ClassificationResult>,
        inference_time_ms: f32,
        preprocessing_time_ms: f32,
        postprocessing_time_ms: f32,
    ) -> Self {
        let total_time_ms = preprocessing_time_ms + inference_time_ms + postprocessing_time_ms;
        Self::new(
            data,
            shape,
            is_classification,
            top_predictions,
            inference_time_ms,
            preprocessing_time_ms,
            postprocessing_time_ms,
            total_time_ms,
        )
    }

    /// Get the number of elements in the output
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Check if the result is empty
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Get the top prediction if available
    pub fn top_prediction(&self) -> Option<&ClassificationResult> {
        self.top_predictions.first()
    }
}

impl fmt::Display for InferenceResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "InferenceResult: {} elements, {:.2}ms total", self.data.len(), self.total_time_ms)?;
        if self.is_classification && !self.top_predictions.is_empty() {
            write!(f, ", Top: {}", self.top_predictions[0])?;
        }
        write!(f, " (prep: {:.2}ms, inference: {:.2}ms, post: {:.2}ms)", 
               self.preprocessing_time_ms, self.inference_time_ms, self.postprocessing_time_ms)?;
        Ok(())
    }
}