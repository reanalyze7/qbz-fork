pub(super) fn is_nvidia_gpu() -> bool {
    std::path::Path::new("/proc/driver/nvidia/version").exists()
        || std::fs::read_to_string("/proc/modules")
            .map(|m| m.lines().any(|l| l.starts_with("nvidia")))
            .unwrap_or(false)
}

pub(super) fn is_amd_gpu() -> bool {
    std::path::Path::new("/sys/module/amdgpu").exists()
        || std::fs::read_to_string("/proc/modules")
            .map(|m| m.lines().any(|l| l.starts_with("amdgpu")))
            .unwrap_or(false)
}

pub(super) fn is_intel_gpu() -> bool {
    std::path::Path::new("/sys/module/i915").exists()
        || std::fs::read_to_string("/proc/modules")
            .map(|m| m.lines().any(|l| l.starts_with("i915")))
            .unwrap_or(false)
}

pub fn detect_gpu_name(nvidia: bool, amd: bool, intel: bool) -> String {
    // Hybrid laptops have more than one of these set; join the names so
    // diagnostics surface the full picture instead of returning only the
    // first vendor matched.
    let mut parts: Vec<String> = Vec::new();
    if nvidia {
        parts.push(nvidia_name());
    }
    if amd {
        parts.push(amd_name());
    }
    if intel {
        parts.push(intel_name());
    }
    if parts.is_empty() {
        "Unknown / None detected".to_string()
    } else {
        parts.join(" + ")
    }
}

fn nvidia_name() -> String {
    if let Ok(version) = std::fs::read_to_string("/proc/driver/nvidia/version") {
        if let Some(line) = version.lines().next() {
            return format!("NVIDIA ({})", line.trim());
        }
    }
    "NVIDIA (driver loaded)".to_string()
}

fn amd_name() -> String {
    if let Ok(entries) = std::fs::read_dir("/sys/class/drm") {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with("card") && !name.contains('-') {
                let model_path = entry.path().join("device/product_name");
                if let Ok(model) = std::fs::read_to_string(&model_path) {
                    let model = model.trim();
                    if !model.is_empty() {
                        return format!("AMD {}", model);
                    }
                }
            }
        }
    }
    "AMD (amdgpu driver loaded)".to_string()
}

fn intel_name() -> String {
    "Intel (i915/xe driver loaded)".to_string()
}
