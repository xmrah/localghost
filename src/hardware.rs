use std::process::Command;
use std::fs;

#[derive(Debug, Clone, Default)]
pub struct HardwareInfo {
    pub gpu_vendor: Option<String>,
    pub cpu_vendor: Option<String>,
}

impl HardwareInfo {
    pub fn to_context_string(&self) -> String {
        let mut parts = vec![];
        match self.gpu_vendor.as_deref() {
            Some("nvidia") => parts.push("GPU: NVIDIA (nvidia-smi kullan)".to_string()),
            Some("amd")    => parts.push("GPU: AMD (radeontop veya rocm-smi kullan, nvidia-smi kullanma)".to_string()),
            Some("intel")  => parts.push("GPU: Intel (intel_gpu_top kullan)".to_string()),
            _ => {}
        }
        if let Some(cpu) = &self.cpu_vendor {
            parts.push(format!("CPU: {}", cpu.to_uppercase()));
        }
        if parts.is_empty() {
            "Bilinmiyor".to_string()
        } else {
            parts.join(". ")
        }
    }
}

pub fn detect() -> HardwareInfo {
    HardwareInfo {
        gpu_vendor: detect_gpu(),
        cpu_vendor: detect_cpu(),
    }
}

fn detect_gpu() -> Option<String> {
    let output = Command::new("lspci")
        .output()
        .ok()?;
    let text = String::from_utf8_lossy(&output.stdout).to_lowercase();

    if text.contains("nvidia") {
        Some("nvidia".to_string())
    } else if text.contains("amd") || text.contains("radeon") {
        Some("amd".to_string())
    } else if text.contains("intel") {
        Some("intel".to_string())
    } else {
        None
    }
}

fn detect_cpu() -> Option<String> {
    let text = fs::read_to_string("/proc/cpuinfo").ok()?.to_lowercase();
    if text.contains("amd") {
        Some("amd".to_string())
    } else if text.contains("intel") {
        Some("intel".to_string())
    } else if text.contains("arm") || text.contains("aarch64") {
        Some("arm".to_string())
    } else {
        None
    }
}
