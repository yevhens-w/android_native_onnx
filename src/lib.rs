use std::ptr;
use std::sync::Mutex;
use jni::JNIEnv;
use jni::objects::{JClass, JString, JByteArray};
use jni::sys::{jfloatArray, jstring, jint, jintArray};
use ort::session::Session;

// Import our modules
mod constants;
mod errors;
mod inference;
mod labels;
mod types;
mod error_helper;

// Re-export types for external use
use crate::inference::InferenceEngine;
use crate::labels::LabelsManager;
use crate::types::InferenceResult;




pub fn run_inference_internal(
    image_bytes: &[u8],
) -> Result<InferenceResult, Box<dyn std::error::Error>> {
    match InferenceEngine::run_inference(image_bytes) {
        Ok(result) => Ok(result),
        Err(e) => {
            let error_msg = e.to_string();
            InferenceEngine::store_error(&error_msg);
            Err(error_msg.into())
        }
    }
}


// Static storage for panic information (JNI specific)
static PANIC_INFO: Mutex<Option<String>> = Mutex::new(None);

// Load model into cache for faster inference
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_example_onnxapp_OnnxInference_loadModelNative(
    mut env: JNIEnv,
    _class: JClass,
    model_path: JString,
) -> jstring {
    let model_path_str: String = match env.get_string(&model_path) {
        Ok(s) => s.into(),
        Err(_) => {
            let error = "Failed to get model path from JNI";
            InferenceEngine::store_error(error);
            return match env.new_string(error) {
                Ok(jstr) => jstr.into_raw(),
                Err(_) => ptr::null_mut(),
            }
        }
    };

    let result = match InferenceEngine::load_model(&model_path_str) {
        Ok(_) => format!("Model loaded successfully: {}", model_path_str),
        Err(e) => {
            let error_msg = format!("Failed to load model: {}", e);
            InferenceEngine::store_error(&error_msg);
            error_msg
        }
    };

    match env.new_string(&result) {
        Ok(jstr) => jstr.into_raw(),
        Err(_) => ptr::null_mut(),
    }
}

// Check if any model is currently loaded in cache
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_example_onnxapp_OnnxInference_isModelLoadedNative(
    _env: JNIEnv,
    _class: JClass,
) -> jint {
    if InferenceEngine::is_model_loaded() { 1 } else { 0 }
}

// Get the path of the currently loaded model
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_example_onnxapp_OnnxInference_getLoadedModelPathNative(
    env: JNIEnv,
    _class: JClass,
) -> jstring {
    match InferenceEngine::get_loaded_model_path() {
        Some(path) => match env.new_string(&path) {
            Ok(jstr) => jstr.into_raw(),
            Err(_) => ptr::null_mut(),
        },
        None => match env.new_string("") {
            Ok(jstr) => jstr.into_raw(),
            Err(_) => ptr::null_mut(),
        }
    }
}

// Get inference time from last run
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_example_onnxapp_OnnxInference_getInferenceTimeNative(
    _env: JNIEnv,
    _class: JClass,
) -> jni::sys::jfloat {
    if let Some(result) = InferenceEngine::get_last_result() {
        result.inference_time_ms
    } else {
        0.0
    }
}

// Get preprocessing time from last run
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_example_onnxapp_OnnxInference_getPreprocessingTimeNative(
    _env: JNIEnv,
    _class: JClass,
) -> jni::sys::jfloat {
    if let Some(result) = InferenceEngine::get_last_result() {
        result.preprocessing_time_ms
    } else {
        0.0
    }
}

// Get postprocessing time from last run
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_example_onnxapp_OnnxInference_getPostprocessingTimeNative(
    _env: JNIEnv,
    _class: JClass,
) -> jni::sys::jfloat {
    if let Some(result) = InferenceEngine::get_last_result() {
        result.postprocessing_time_ms
    } else {
        0.0
    }
}

// Get total time from last run
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_example_onnxapp_OnnxInference_getTotalTimeNative(
    _env: JNIEnv,
    _class: JClass,
) -> jni::sys::jfloat {
    if let Some(result) = InferenceEngine::get_last_result() {
        result.total_time_ms
    } else {
        0.0
    }
}

// Test function to verify JNI is working
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_example_onnxapp_OnnxInference_testJNINative(
    env: JNIEnv,
    _class: JClass,
) -> jstring {
    match env.new_string("JNI is working!") {
        Ok(jstr) => jstr.into_raw(),
        Err(_) => ptr::null_mut(),
    }
}

// Test function to check basic image processing without ONNX
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_example_onnxapp_OnnxInference_testImageProcessing(
    mut env: JNIEnv,
    _class: JClass,
    image_path: JString,
) -> jstring {
    let image_path_str = match env.get_string(&image_path) {
        Ok(s) => s,
        Err(_) => {
            return match env.new_string("Failed to get image path") {
                Ok(jstr) => jstr.into_raw(),
                Err(_) => ptr::null_mut(),
            }
        }
    };

    let image_path: String = image_path_str.into();

    // Try to load image without ONNX Runtime
    let result = match image::open(&image_path) {
        Ok(img) => {
            let width = img.width();
            let height = img.height();
            format!("Image loaded successfully: {}x{}", width, height)
        }
        Err(e) => format!("Failed to load image: {}", e),
    };

    match env.new_string(&result) {
        Ok(jstr) => jstr.into_raw(),
        Err(_) => ptr::null_mut(),
    }
}

// Test ONNX Runtime initialization only (no model loading)
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_example_onnxapp_OnnxInference_testOnnxInit(
    env: JNIEnv,
    _class: JClass,
) -> jstring {
    // Step 1: Safe environment variable checking
    let result = match std::panic::catch_unwind(|| {
        let mut debug_info = String::new();
        
        // Check library paths
        if let Ok(ld_library_path) = std::env::var("LD_LIBRARY_PATH") {
            debug_info.push_str(&format!("LD_LIBRARY_PATH: {}\n", ld_library_path));
        } else {
            debug_info.push_str("LD_LIBRARY_PATH: not set\n");
        }
        
        debug_info
    }) {
        Ok(debug_info) => {
            // Step 2: Safe file system checking
            let file_check_result = match std::panic::catch_unwind(|| {
                let mut file_info = String::new();
                
                // Static linking - ONNX Runtime is compiled into our library
                file_info.push_str("✅ Using static ONNX Runtime linking\n");
                file_info.push_str("ONNX Runtime is statically compiled into libonnx_inference.so\n");
                file_info.push_str("No separate libonnxruntime.so file needed!\n");
                
                file_info
            }) {
                Ok(file_info) => file_info,
                Err(_) => "File system check panicked!\n".to_string(),
            };
            
            // Step 3: Set up detailed panic hook and try ONNX Runtime initialization
            let original_hook = std::panic::take_hook();
            std::panic::set_hook(Box::new(move |panic_info| {
                let mut panic_msg = String::new();
                panic_msg.push_str("🔥 PANIC HOOK TRIGGERED 🔥\n");
                
                if let Some(location) = panic_info.location() {
                    panic_msg.push_str(&format!("Location: {}:{}:{}\n", 
                        location.file(), location.line(), location.column()));
                }
                
                if let Some(msg) = panic_info.payload().downcast_ref::<&str>() {
                    panic_msg.push_str(&format!("Message: {}\n", msg));
                } else if let Some(msg) = panic_info.payload().downcast_ref::<String>() {
                    panic_msg.push_str(&format!("Message: {}\n", msg));
                } else {
                    panic_msg.push_str("Message: (non-string panic payload)\n");
                }
                
                // Store panic info globally
                if let Ok(mut panic_storage) = PANIC_INFO.lock() {
                    *panic_storage = Some(panic_msg.clone());
                }
                
                // Call original hook
                original_hook(panic_info);
            }));
            
            let session_result = match std::panic::catch_unwind(|| {
                // Add environment info that might be relevant to ONNX Runtime
                let mut env_info = String::new();
                env_info.push_str("Environment variables:\n");
                
                // Check common variables that might affect ONNX Runtime
                let vars_to_check = [
                    "LD_LIBRARY_PATH", "ANDROID_DATA", "ANDROID_ROOT", "PATH",
                    "ORT_DYLIB_PATH", "OMP_NUM_THREADS", "CUDA_VISIBLE_DEVICES"
                ];
                
                for var in &vars_to_check {
                    match std::env::var(var) {
                        Ok(value) => env_info.push_str(&format!("  {}: {}\n", var, value)),
                        Err(_) => env_info.push_str(&format!("  {}: (not set)\n", var)),
                    }
                }
                
                // Add thread info
                let thread_id = std::thread::current().id();
                env_info.push_str(&format!("Thread ID: {:?}\n", thread_id));
                
                // Add process info if available
                if let Ok(pid) = std::process::id().to_string().parse::<u32>() {
                    env_info.push_str(&format!("Process ID: {}\n", pid));
                }
                
                env_info.push_str("\nTrying ONNX Runtime Session::builder()...\n");
                // Try the original Session::builder() approach
                match Session::builder() {
                    Ok(_builder) => {
                        env_info.push_str("✅ ONNX Runtime Session::builder() succeeded!\n");
                        format!("✅ ONNX Runtime SUCCESS!\n{}", env_info)
                    }
                    Err(e) => {
                        env_info.push_str(&format!("❌ ONNX Runtime Session::builder() failed: {:?}\n", e));
                        env_info.push_str("\n💡 Common solutions:\n");
                        env_info.push_str("- Check ONNX Runtime library is properly linked\n");
                        env_info.push_str("- Verify Android architecture compatibility\n");
                        env_info.push_str("- Try TensorFlow Lite as alternative\n");
                        format!("❌ ONNX Runtime ERROR:\n{}", env_info)
                    }
                }
            }) {
                Ok(result) => result,
                Err(panic_info) => {
                    // Extract detailed panic information
                    let mut panic_details = String::new();
                    panic_details.push_str("💥 COMPREHENSIVE PANIC ANALYSIS:\n");
                    panic_details.push_str("==================================\n\n");
                    
                    // Include panic hook information if available
                    if let Ok(panic_storage) = PANIC_INFO.lock() {
                        if let Some(ref hook_info) = *panic_storage {
                            panic_details.push_str("PANIC HOOK DATA:\n");
                            panic_details.push_str(hook_info);
                            panic_details.push_str("\n");
                        }
                    }
                    
                    // Try to extract panic message from catch_unwind
                    panic_details.push_str("CATCH_UNWIND DATA:\n");
                    let panic_message = if let Some(s) = panic_info.downcast_ref::<&str>() {
                        format!("Panic payload (str): {}", s)
                    } else if let Some(s) = panic_info.downcast_ref::<String>() {
                        format!("Panic payload (String): {}", s)
                    } else {
                        "Panic payload: (custom type, not extractable as string)".to_string()
                    };
                    panic_details.push_str(&format!("{}\n\n", panic_message));
                    
                    // Add context information
                    panic_details.push_str("CONTEXT:\n");
                    panic_details.push_str(&format!("Thread: {:?}\n", std::thread::current().id()));
                    panic_details.push_str("Function: Session::builder()\n");
                    panic_details.push_str("Library: ONNX Runtime (ort crate)\n");
                    panic_details.push_str("Platform: Android\n\n");
                    
                    // Add diagnostic information
                    panic_details.push_str("DIAGNOSIS:\n");
                    panic_details.push_str("This panic occurs during ONNX Runtime initialization.\n");
                    panic_details.push_str("Most likely causes:\n");
                    panic_details.push_str("  1. libonnxruntime.so loading failure\n");
                    panic_details.push_str("  2. Missing system dependencies (libc++, libdl, libm)\n");
                    panic_details.push_str("  3. Android API compatibility issues\n");
                    panic_details.push_str("  4. CPU architecture mismatch\n");
                    panic_details.push_str("  5. Memory allocation failures\n");
                    panic_details.push_str("  6. Symbol resolution failures\n\n");
                    
                    // Reset panic hook
                    let _ = std::panic::take_hook();
                    
                    panic_details
                }
            };
            
            format!("Debug Info:\n{}\nFile Check:\n{}\nSession Builder:\n{}", 
                    debug_info, file_check_result, session_result)
        }
        Err(_) => {
            "Environment variable check panicked!".to_string()
        }
    };
    
    // Store the result/error for retrieval  
    InferenceEngine::store_error(&result);
    
    match env.new_string(&result) {
        Ok(jstr) => jstr.into_raw(),
        Err(_) => ptr::null_mut(),
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_example_onnxapp_OnnxInference_runInferenceNative(
    env: JNIEnv,
    _class: JClass,
    image_bytes: JByteArray,
) -> jfloatArray {
    // Get byte array from Java
    let image_data = match env.convert_byte_array(image_bytes) {
        Ok(data) => data,
        Err(_) => return ptr::null_mut(),
    };
    
    // Use the converted Vec<i8> and convert to &[u8]
    let image_slice: &[u8] = unsafe {
        std::slice::from_raw_parts(image_data.as_ptr() as *const u8, image_data.len())
    };

    // Create a debug log function for Android
    let log_debug = |msg: &str| {
        if let Ok(jstr) = env.new_string(&format!("RUST_DEBUG: {}", msg)) {
            // Try to log via Android's logging system if available
            // For now, we'll just ignore logging errors
            let _ = jstr;
        }
    };

    match run_inference_internal(image_slice) {
        Ok(result) => {
            log_debug(&format!("Inference successful, data size: {}", result.data.len()));
            
            match env.new_float_array(result.data.len() as jint) {
                Ok(array) => {
                    if env.set_float_array_region(&array, 0, &result.data).is_ok() {
                        log_debug("Successfully created and populated float array");
                        array.into_raw()
                    } else {
                        log_debug("Failed to set float array region");
                        ptr::null_mut()
                    }
                }
                Err(e) => {
                    log_debug(&format!("Failed to create float array: {:?}", e));
                    ptr::null_mut()
                }
            }
        }
        Err(e) => {
            let error_msg = format!("Inference failed: {}", e);
            log_debug(&error_msg);

            // Error is already stored by run_inference_internal

            ptr::null_mut()
        }
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_example_onnxapp_OnnxInference_isClassificationNative(
    _env: JNIEnv,
    _class: JClass,
) -> jint {
    if let Some(result) = InferenceEngine::get_last_result() {
        return if result.is_classification { 1 } else { 0 };
    }
    0
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_example_onnxapp_OnnxInference_getOutputShapeNative(
    env: JNIEnv,
    _class: JClass,
) -> jintArray {
    if let Some(result) = InferenceEngine::get_last_result() {
        let shape_i32: Vec<jint> = result.shape.iter().map(|&x| x as jint).collect();
        match env.new_int_array(shape_i32.len() as jint) {
            Ok(array) => {
                if env.set_int_array_region(&array, 0, &shape_i32).is_ok() {
                    return array.into_raw();
                }
            }
            Err(_) => {}
        }
    }
    ptr::null_mut()
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_example_onnxapp_OnnxInference_getTopPredictionsJsonNative(
    env: JNIEnv,
    _class: JClass,
) -> jstring {
    if let Some(result) = InferenceEngine::get_last_result() {
        if result.is_classification && !result.top_predictions.is_empty() {
            // Create JSON string with predictions
            let mut json_parts = Vec::new();
            for prediction in &result.top_predictions {
                json_parts.push(format!(
                    "{{\"class_id\":{},\"class_name\":\"{}\",\"confidence\":{}}}",
                    prediction.class_id,
                    prediction.class_name.replace('"', "\\\""),
                    prediction.confidence
                ));
            }
            let json = format!("[{}]", json_parts.join(","));

            match env.new_string(&json) {
                Ok(jstr) => return jstr.into_raw(),
                Err(_) => {}
            }
        }
    }
    ptr::null_mut()
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_example_onnxapp_OnnxInference_loadImageNetLabelsNative(
    mut env: JNIEnv,
    _class: JClass,
    labels_path: JString,
) -> jstring {
    let labels_path_str: String = match env.get_string(&labels_path) {
        Ok(s) => s.into(),
        Err(_) => {
            return match env.new_string("Failed to get labels path from JNI") {
                Ok(jstr) => jstr.into_raw(),
                Err(_) => ptr::null_mut(),
            }
        }
    };

    let result = match LabelsManager::load_labels_from_file(&labels_path_str) {
        Ok(count) => format!("Successfully loaded {} ImageNet labels", count),
        Err(e) => e.to_string()
    };

    match env.new_string(&result) {
        Ok(jstr) => jstr.into_raw(),
        Err(_) => ptr::null_mut(),
    }
}

