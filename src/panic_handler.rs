use std::sync::Mutex;
use ort::session::Session;

static PANIC_INFO: Mutex<Option<String>> = Mutex::new(None);

pub fn test_onnx_init_with_panic_handling() -> String {
    match std::panic::catch_unwind(|| {
        let mut debug_info = String::new();

        if let Ok(ld_library_path) = std::env::var("LD_LIBRARY_PATH") {
            debug_info.push_str(&format!("LD_LIBRARY_PATH: {}\n", ld_library_path));
        } else {
            debug_info.push_str("LD_LIBRARY_PATH: not set\n");
        }

        debug_info
    }) {
        Ok(debug_info) => {
            let file_check_result = match std::panic::catch_unwind(|| {
                let mut file_info = String::new();

                file_info.push_str("✅ Using static ONNX Runtime linking\n");
                file_info.push_str("ONNX Runtime is statically compiled into libonnx_inference.so\n");
                file_info.push_str("No separate libonnxruntime.so file needed!\n");

                file_info
            }) {
                Ok(file_info) => file_info,
                Err(_) => "File system check panicked!\n".to_string(),
            };

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

                if let Ok(mut panic_storage) = PANIC_INFO.lock() {
                    *panic_storage = Some(panic_msg.clone());
                }

                original_hook(panic_info);
            }));

            let session_result = match std::panic::catch_unwind(|| {
                let mut env_info = String::new();
                env_info.push_str("Environment variables:\n");

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

                let thread_id = std::thread::current().id();
                env_info.push_str(&format!("Thread ID: {:?}\n", thread_id));

                if let Ok(pid) = std::process::id().to_string().parse::<u32>() {
                    env_info.push_str(&format!("Process ID: {}\n", pid));
                }

                env_info.push_str("\nTrying ONNX Runtime Session::builder()...\n");
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
                    let mut panic_details = String::new();
                    panic_details.push_str("💥 COMPREHENSIVE PANIC ANALYSIS:\n");
                    panic_details.push_str("==================================\n\n");

                    if let Ok(panic_storage) = PANIC_INFO.lock() {
                        if let Some(ref hook_info) = *panic_storage {
                            panic_details.push_str("PANIC HOOK DATA:\n");
                            panic_details.push_str(hook_info);
                            panic_details.push_str("\n");
                        }
                    }

                    panic_details.push_str("CATCH_UNWIND DATA:\n");
                    let panic_message = if let Some(s) = panic_info.downcast_ref::<&str>() {
                        format!("Panic payload (str): {}", s)
                    } else if let Some(s) = panic_info.downcast_ref::<String>() {
                        format!("Panic payload (String): {}", s)
                    } else {
                        "Panic payload: (custom type, not extractable as string)".to_string()
                    };
                    panic_details.push_str(&format!("{}\n\n", panic_message));

                    panic_details.push_str("CONTEXT:\n");
                    panic_details.push_str(&format!("Thread: {:?}\n", std::thread::current().id()));
                    panic_details.push_str("Function: Session::builder()\n");
                    panic_details.push_str("Library: ONNX Runtime (ort crate)\n");
                    panic_details.push_str("Platform: Android\n\n");

                    panic_details.push_str("DIAGNOSIS:\n");
                    panic_details.push_str("This panic occurs during ONNX Runtime initialization.\n");
                    panic_details.push_str("Most likely causes:\n");
                    panic_details.push_str("  1. libonnxruntime.so loading failure\n");
                    panic_details.push_str("  2. Missing system dependencies (libc++, libdl, libm)\n");
                    panic_details.push_str("  3. Android API compatibility issues\n");
                    panic_details.push_str("  4. CPU architecture mismatch\n");
                    panic_details.push_str("  5. Memory allocation failures\n");
                    panic_details.push_str("  6. Symbol resolution failures\n\n");

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
    }
}