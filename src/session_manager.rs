use crate::errors::{InferenceError, InferenceResult};
use ort::session::Session;
use std::collections::HashMap;
use std::sync::Mutex;

static CACHED_SESSIONS: std::sync::LazyLock<Mutex<HashMap<String, Session>>> = std::sync::LazyLock::new(|| {
    Mutex::new(HashMap::new())
});

static CURRENT_MODEL: Mutex<Option<String>> = Mutex::new(None);

static EXECUTION_PROVIDER_INFO: Mutex<Option<String>> = Mutex::new(None);

pub struct SessionManager;

impl SessionManager {
    pub fn try_create_gpu_session(model_bytes: &[u8]) -> Result<Session, Box<dyn std::error::Error>> {
        Session::builder()?
            .commit_from_memory(model_bytes)
            .map_err(|e| e.into())
    }

    pub fn load_model(model_path: &str) -> InferenceResult<()> {
        if !std::path::Path::new(model_path).exists() {
            return Err(InferenceError::model_not_found(model_path));
        }

        {
            if let Ok(cached_sessions) = CACHED_SESSIONS.lock() {
                if cached_sessions.contains_key(model_path) {
                    return Ok(());
                }
            }
        }

        let model_bytes = std::fs::read(model_path)
            .map_err(|e| InferenceError::model_loading_failed(format!("Failed to read model file {}: {}", model_path, e)))?;

        let session = match Self::try_create_gpu_session(&model_bytes) {
            Ok(session) => session,
            Err(_) => {
                Session::builder()
                    .map_err(|e| InferenceError::session_failed(format!("Failed to create ONNX session builder: {:?}", e)))?
                    .commit_from_memory(&model_bytes)
                    .map_err(|e| InferenceError::model_loading_failed(format!("Failed to load model from memory: {:?}", e)))?
            }
        };

        let provider_info = Self::detect_execution_provider_info(&session);
        if let Ok(mut exec_info) = EXECUTION_PROVIDER_INFO.lock() {
            *exec_info = Some(provider_info);
        }

        if let Ok(mut cached_sessions) = CACHED_SESSIONS.lock() {
            cached_sessions.insert(model_path.to_string(), session);
        } else {
            return Err(InferenceError::memory_error("Failed to acquire session cache mutex"));
        }

        Ok(())
    }

    pub fn get_session(model_path: &str) -> InferenceResult<std::sync::MutexGuard<'_, HashMap<String, Session>>> {
        let cached_sessions = CACHED_SESSIONS.lock()
            .map_err(|_| InferenceError::memory_error("Failed to acquire sessions cache mutex"))?;

        if !cached_sessions.contains_key(model_path) {
            return Err(InferenceError::model_not_found(&format!("Model not found in cache: {}. Call load_model first.", model_path)));
        }

        Ok(cached_sessions)
    }

    pub fn is_model_loaded() -> bool {
        if let Ok(current_model) = CURRENT_MODEL.lock() {
            if let Some(ref model_path) = *current_model {
                if let Ok(cached_sessions) = CACHED_SESSIONS.lock() {
                    return cached_sessions.contains_key(model_path);
                }
            }
        }
        false
    }

    pub fn get_current_model_path() -> Option<String> {
        CURRENT_MODEL.lock().ok()?.clone()
    }

    pub fn set_current_model(model_path: &str) -> InferenceResult<()> {
        {
            if let Ok(cached_sessions) = CACHED_SESSIONS.lock() {
                if !cached_sessions.contains_key(model_path) {
                    return Err(InferenceError::model_not_found(&format!("Model not in cache: {}. Call load_model first.", model_path)));
                }
            } else {
                return Err(InferenceError::memory_error("Failed to acquire sessions cache mutex"));
            }
        }

        if let Ok(mut current_model) = CURRENT_MODEL.lock() {
            *current_model = Some(model_path.to_string());
            Ok(())
        } else {
            Err(InferenceError::memory_error("Failed to acquire current model mutex"))
        }
    }

    pub fn get_execution_provider_info() -> Option<String> {
        EXECUTION_PROVIDER_INFO.lock().ok()?.as_ref().cloned()
    }

    fn detect_execution_provider_info(_session: &Session) -> String {
        let mut available_providers: Vec<String> = Vec::new();

        #[cfg(target_os = "android")]
        {
            if std::fs::metadata("/apex/com.android.neuralnetworks/lib64/libneuralnetworks.so").is_ok() ||
               std::fs::metadata("/system/lib64/libneuralnetworks.so").is_ok() {
                available_providers.push("NNAPI (Android NPU/DSP)".to_string());
            }

            if std::fs::metadata("/system/lib64/libOpenCL.so").is_ok() ||
               std::fs::metadata("/vendor/lib64/libOpenCL.so").is_ok() ||
               std::fs::metadata("/system/lib/libOpenCL.so").is_ok() {
                available_providers.push("OpenCL (Android GPU)".to_string());
            }

            if std::fs::metadata("/vendor/lib64/libvulkan.so").is_ok() ||
               std::fs::metadata("/system/lib64/libvulkan.so").is_ok() {
                available_providers.push("Vulkan (Android GPU)".to_string());
            }

            if std::fs::metadata("/vendor/lib64/libadreno_utils.so").is_ok() {
                available_providers.push("Adreno GPU".to_string());
            }
        }

        if std::env::var("CUDA_VISIBLE_DEVICES").is_ok() {
            available_providers.push("CUDA (GPU)".to_string());
        }

        available_providers.push("CPU".to_string());

        let active_provider = if available_providers.len() > 1 && !available_providers[0].contains("CPU") {
            available_providers[0].clone()
        } else if available_providers.len() > 1 {
            available_providers.iter()
                .find(|p| p.contains("GPU") || p.contains("NPU") || p.contains("DSP"))
                .unwrap_or(&available_providers[0])
                .clone()
        } else {
            "CPU (GPU libs may not be available)".to_string()
        };

        let gpu_note = if available_providers.iter().any(|p| p.contains("GPU") || p.contains("NPU") || p.contains("DSP")) {
            " 🚀"
        } else {
            " (CPU only - install GPU runtime for acceleration)"
        };

        if available_providers.len() > 1 {
            format!("Available: {} | Active: {}{}", available_providers.join(", "), active_provider, gpu_note)
        } else {
            format!("Active: {}{}", active_provider, gpu_note)
        }
    }
}