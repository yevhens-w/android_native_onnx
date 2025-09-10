// Get last error message for debugging
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_example_onnxapp_OnnxInference_getLastError(
    env: jni::JNIEnv,
    _class: jni::objects::JClass,
) -> jni::sys::jstring {
    use std::ptr;
    use crate::inference::InferenceEngine;
    
    if let Some(error) = InferenceEngine::get_last_error() {
        match env.new_string(&error) {
            Ok(jstr) => return jstr.into_raw(),
            Err(_) => {}
        }
    }
    
    match env.new_string("No error message available") {
        Ok(jstr) => jstr.into_raw(),
        Err(_) => ptr::null_mut(),
    }
}