use std::sync::Mutex;

static MEMORY_USAGE_INFO: Mutex<Option<String>> = Mutex::new(None);

pub struct SystemInfo;

impl SystemInfo {
    pub fn get_memory_usage_info() -> Option<String> {
        Self::update_memory_usage();
        MEMORY_USAGE_INFO.lock().ok()?.as_ref().cloned()
    }

    fn update_memory_usage() {
        let memory_info = Self::calculate_memory_usage();
        if let Ok(mut mem_info) = MEMORY_USAGE_INFO.lock() {
            *mem_info = Some(memory_info);
        }
    }

    fn calculate_memory_usage() -> String {
        let mut info_parts = Vec::new();

        #[cfg(target_os = "android")]
        {
            if let Some(system_memory) = Self::get_android_system_memory() {
                info_parts.push(system_memory);
            }
        }

        let process_memory = Self::get_process_memory();
        if !process_memory.is_empty() {
            info_parts.push(format!("Process: {}", process_memory));
        }

        let session_info = Self::estimate_session_memory();
        if !session_info.is_empty() {
            info_parts.push(session_info);
        }

        if info_parts.is_empty() {
            "Memory info unavailable".to_string()
        } else {
            info_parts.join(" | ")
        }
    }

    #[cfg(target_os = "android")]
    fn get_android_system_memory() -> Option<String> {
        let meminfo = std::fs::read_to_string("/proc/meminfo").ok()?;
        let mut total_kb = 0;
        let mut available_kb = 0;

        for line in meminfo.lines() {
            if line.starts_with("MemTotal:") {
                if let Some(kb_str) = line.split_whitespace().nth(1) {
                    total_kb = kb_str.parse::<u64>().unwrap_or(0);
                }
            } else if line.starts_with("MemAvailable:") {
                if let Some(kb_str) = line.split_whitespace().nth(1) {
                    available_kb = kb_str.parse::<u64>().unwrap_or(0);
                }
            }
        }

        if total_kb > 0 && available_kb > 0 {
            let used_kb = total_kb - available_kb;
            let used_mb = used_kb / 1024;
            let total_mb = total_kb / 1024;
            let available_mb = available_kb / 1024;

            Some(format!("System: {}MB used / {}MB total ({}MB available)",
                used_mb, total_mb, available_mb))
        } else {
            None
        }
    }

    #[cfg(not(target_os = "android"))]
    #[allow(dead_code)]
    fn get_android_system_memory() -> Option<String> {
        None
    }

    fn get_process_memory() -> String {
        if let Ok(status) = std::fs::read_to_string("/proc/self/status") {
            let mut vm_rss_kb = 0;
            let mut vm_size_kb = 0;

            for line in status.lines() {
                if line.starts_with("VmRSS:") {
                    if let Some(kb_str) = line.split_whitespace().nth(1) {
                        vm_rss_kb = kb_str.parse::<u64>().unwrap_or(0);
                    }
                } else if line.starts_with("VmSize:") {
                    if let Some(kb_str) = line.split_whitespace().nth(1) {
                        vm_size_kb = kb_str.parse::<u64>().unwrap_or(0);
                    }
                }
            }

            if vm_rss_kb > 0 {
                let rss_mb = vm_rss_kb / 1024;
                let size_mb = vm_size_kb / 1024;
                return format!("{}MB RSS / {}MB virtual", rss_mb, size_mb);
            }
        }

        "Unknown".to_string()
    }

    fn estimate_session_memory() -> String {
        // This would need to be updated to work with the session manager
        // For now, return empty string
        String::new()
    }
}