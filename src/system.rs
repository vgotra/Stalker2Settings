use sysinfo::{System, SystemExt, ComponentExt, DiskExt, CpuExt};
use crate::models::SystemInfo;

/// Get system information for settings recommendations
pub fn get_system_info() -> SystemInfo {
    let mut sys = System::new_all();
    sys.refresh_all();

    // Get CPU information
    let cpu_name = if !sys.cpus().is_empty() {
        sys.cpus()[0].brand().to_string()
    } else {
        "Unknown CPU".to_string()
    };

    let cpu_cores = sys.cpus().len() as u32;

    // Get RAM information
    let ram_gb = sys.total_memory() / 1024 / 1024; // Convert from KB to GB

    // Get GPU information (this is limited in sysinfo)
    // For more accurate GPU detection, we might need platform-specific code
    let mut gpu_name = "Unknown GPU".to_string();
    let mut gpu_vram_mb = 2048; // Default to 2GB if we can't detect

    // Try to get GPU info from components
    for component in sys.components() {
        let label = component.label().to_lowercase();
        if label.contains("gpu") || label.contains("graphics") {
            gpu_name = component.label().to_string();
            break;
        }
    }

    // On Windows, we could use DXGI or WMI to get better GPU info
    // On Linux, we could parse lspci output
    // For now, we'll use a simple heuristic based on total memory
    // More RAM often correlates with better GPUs
    if ram_gb >= 32 {
        gpu_vram_mb = 8192; // Assume 8GB VRAM for systems with 32GB+ RAM
    } else if ram_gb >= 16 {
        gpu_vram_mb = 6144; // Assume 6GB VRAM for systems with 16GB RAM
    } else if ram_gb >= 8 {
        gpu_vram_mb = 4096; // Assume 4GB VRAM for systems with 8GB RAM
    }

    SystemInfo {
        cpu_name,
        cpu_cores,
        ram_gb,
        gpu_name,
        gpu_vram_mb,
    }
}

/// Detect if the system meets minimum requirements
pub fn meets_minimum_requirements(info: &SystemInfo) -> bool {
    // Minimum requirements for Stalker 2:
    // - CPU: AMD Ryzen 5 1600X / Intel Core i5-7600K
    // - RAM: 8 GB
    // - GPU: AMD Radeon RX 580 8GB / NVIDIA GeForce GTX 1060 6GB
    // - VRAM: 6 GB

    // Simple checks (these are approximations)
    let has_enough_cores = info.cpu_cores >= 4;
    let has_enough_ram = info.ram_gb >= 8;
    let has_enough_vram = info.gpu_vram_mb >= 6 * 1024;

    has_enough_cores && has_enough_ram && has_enough_vram
}

/// Detect if the system meets recommended requirements
pub fn meets_recommended_requirements(info: &SystemInfo) -> bool {
    // Recommended requirements for Stalker 2:
    // - CPU: AMD Ryzen 7 3700X / Intel Core i7-9700K
    // - RAM: 16 GB
    // - GPU: AMD Radeon RX 5700 XT / NVIDIA GeForce RTX 2070 SUPER
    // - VRAM: 8 GB

    // Simple checks (these are approximations)
    let has_enough_cores = info.cpu_cores >= 8;
    let has_enough_ram = info.ram_gb >= 16;
    let has_enough_vram = info.gpu_vram_mb >= 8 * 1024;

    has_enough_cores && has_enough_ram && has_enough_vram
}

/// Get a performance tier based on system specs
pub fn get_performance_tier(info: &SystemInfo) -> &'static str {
    if meets_recommended_requirements(info) {
        "High"
    } else if meets_minimum_requirements(info) {
        "Medium"
    } else {
        "Low"
    }
}
