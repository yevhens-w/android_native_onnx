use std::ptr;
use jni::JNIEnv;
use jni::objects::{JClass, JString, JByteArray};
use jni::sys::{jfloatArray, jstring, jint, jintArray};

use crate::inference::InferenceEngine;
use crate::labels::LabelsManager;
use crate::run_inference_internal;
use crate::panic_handler::test_onnx_init_with_panic_handling;
use crate::session_manager::SessionManager;
use crate::system_info::SystemInfo;

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

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_example_onnxapp_OnnxInference_isModelLoadedNative(
    _env: JNIEnv,
    _class: JClass,
) -> jint {
    if InferenceEngine::is_model_loaded() { 1 } else { 0 }
}

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

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_example_onnxapp_OnnxInference_testOnnxInit(
    env: JNIEnv,
    _class: JClass,
) -> jstring {
    let result = test_onnx_init_with_panic_handling();
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
    let image_data = match env.convert_byte_array(image_bytes) {
        Ok(data) => data,
        Err(_) => return ptr::null_mut(),
    };

    let image_slice: &[u8] = unsafe {
        std::slice::from_raw_parts(image_data.as_ptr() as *const u8, image_data.len())
    };

    let log_debug = |msg: &str| {
        if let Ok(jstr) = env.new_string(&format!("RUST_DEBUG: {}", msg)) {
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

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_example_onnxapp_OnnxInference_setCurrentModelNative(
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

    let result = match InferenceEngine::set_current_model(&model_path_str) {
        Ok(_) => format!("Current model set to: {}", model_path_str),
        Err(e) => {
            let error_msg = format!("Failed to set current model: {}", e);
            InferenceEngine::store_error(&error_msg);
            error_msg
        }
    };

    match env.new_string(&result) {
        Ok(jstr) => jstr.into_raw(),
        Err(_) => ptr::null_mut(),
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_example_onnxapp_OnnxInference_getExecutionProviderInfoNative(
    env: JNIEnv,
    _class: JClass,
) -> jstring {
    let info = match SessionManager::get_execution_provider_info() {
        Some(info) => info,
        None => "No execution provider info available".to_string(),
    };

    match env.new_string(&info) {
        Ok(jstr) => jstr.into_raw(),
        Err(_) => ptr::null_mut(),
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_example_onnxapp_OnnxInference_getMemoryUsageInfoNative(
    env: JNIEnv,
    _class: JClass,
) -> jstring {
    let info = match SystemInfo::get_memory_usage_info() {
        Some(info) => info,
        None => "No memory usage info available".to_string(),
    };

    match env.new_string(&info) {
        Ok(jstr) => jstr.into_raw(),
        Err(_) => ptr::null_mut(),
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_example_onnxapp_OnnxInference_getLastError(
    env: JNIEnv,
    _class: JClass,
) -> jstring {
    match InferenceEngine::get_last_error() {
        Some(error) => match env.new_string(&error) {
            Ok(jstr) => jstr.into_raw(),
            Err(_) => ptr::null_mut(),
        },
        None => match env.new_string("") {
            Ok(jstr) => jstr.into_raw(),
            Err(_) => ptr::null_mut(),
        }
    }
}