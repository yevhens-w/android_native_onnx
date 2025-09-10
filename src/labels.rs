/// ImageNet labels management and storage
use crate::constants::FALLBACK_LABELS;
use crate::errors::{InferenceError, InferenceResult};
use std::sync::Mutex;

/// Static storage for ImageNet labels
static IMAGENET_LABELS: Mutex<Option<Vec<String>>> = Mutex::new(None);

/// Labels manager for ImageNet classification
pub struct LabelsManager;

impl LabelsManager {
    /// Get ImageNet labels, falling back to hardcoded labels if not loaded
    pub fn get_labels() -> Vec<String> {
        // Try to get labels from static storage first
        if let Ok(labels_guard) = IMAGENET_LABELS.lock() {
            if let Some(ref labels) = *labels_guard {
                return labels.clone();
            }
        }
        
        // Fallback to hardcoded labels with generated classes for missing ones
        let mut labels = FALLBACK_LABELS.iter().map(|&s| s.to_string()).collect::<Vec<_>>();
        
        // Generate remaining classes up to 1000
        for i in labels.len()..1000 {
            labels.push(format!("class_{}", i));
        }
        
        labels
    }

    /// Load labels from file content
    pub fn load_labels_from_content(content: &str) -> InferenceResult<usize> {
        let labels: Vec<String> = content
            .lines()
            .filter(|line| !line.trim().is_empty())
            .map(|line| line.trim().to_string())
            .collect();

        if labels.is_empty() {
            return Err(InferenceError::labels_loading_failed("Labels file is empty"));
        }

        let count = labels.len();

        // Store labels in static variable
        match IMAGENET_LABELS.lock() {
            Ok(mut labels_guard) => {
                *labels_guard = Some(labels);
                Ok(count)
            }
            Err(_) => Err(InferenceError::labels_loading_failed("Failed to acquire labels mutex")),
        }
    }

    /// Load labels from file path
    pub fn load_labels_from_file(path: &str) -> InferenceResult<usize> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| InferenceError::labels_loading_failed(format!("Failed to read file '{}': {}", path, e)))?;
        
        Self::load_labels_from_content(&content)
    }

    /// Get label for specific class index
    pub fn get_label(index: usize) -> String {
        let labels = Self::get_labels();
        if index < labels.len() {
            labels[index].clone()
        } else {
            format!("class_{}", index)
        }
    }


    /// Clear loaded labels (mainly for testing)
    #[cfg(test)]
    pub fn clear_labels() {
        if let Ok(mut labels_guard) = IMAGENET_LABELS.lock() {
            *labels_guard = None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fallback_labels() {
        LabelsManager::clear_labels();
        let labels = LabelsManager::get_labels();
        assert_eq!(labels.len(), 1000);
        assert_eq!(labels[0], "tench");
        assert_eq!(labels[14], "indigo bunting");
        assert_eq!(labels[999], "class_999");
    }

    #[test]
    fn test_load_labels_from_content() {
        let content = "dog\ncat\nbird\n";
        let result = LabelsManager::load_labels_from_content(content);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 3);

        let labels = LabelsManager::get_labels();
        assert_eq!(labels[0], "dog");
        assert_eq!(labels[1], "cat");
        assert_eq!(labels[2], "bird");
    }

    #[test]
    fn test_empty_content() {
        let content = "\n\n\n";
        let result = LabelsManager::load_labels_from_content(content);
        assert!(result.is_err());
    }
}