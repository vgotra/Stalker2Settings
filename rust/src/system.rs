use sysinfo::{System, SystemExt, ComponentExt, CpuExt};
use crate::models::SystemInfo;
use std::process::Command;
use std::str;

pub fn get_system_info() -> SystemInfo {
    let mut sys = System::new_all();
    sys.refresh_all();

    let cpu_name = if !sys.cpus().is_empty() {
        String::from(sys.cpus()[0].brand())
    } else {
        String::from("Unknown CPU")
    };

    let cpu_cores = sys.cpus().len() as u32;
    let ram_gb = sys.total_memory() / 1024 / 1024;

    let (gpu_name, gpu_vram_mb) = detect_gpu_info(&mut sys);

    SystemInfo {
        cpu_name,
        cpu_cores,
        ram_gb,
        gpu_name,
        gpu_vram_mb,
    }
}

fn detect_gpu_info(sys: &mut System) -> (String, u64) {
    let mut gpu_name = String::from("Unknown GPU");
    let mut gpu_vram_mb = 2048;

    #[cfg(target_os = "windows")]
    {
        if let Ok(output) = Command::new("wmic")
            .args(["path", "win32_VideoController", "get", "name,AdapterRAM", "/format:csv"])
            .output()
        {
            if let Ok(output_str) = str::from_utf8(&output.stdout) {
                let lines: Vec<&str> = output_str.lines().collect();
                if lines.len() > 1 {
                    let parts: Vec<&str> = lines[1].split(',').collect();
                    if parts.len() >= 3 {
                        gpu_name = String::from(parts[1].trim());
                        if let Ok(vram) = parts[2].trim().parse::<u64>() {
                            gpu_vram_mb = vram / (1024 * 1024);
                            return (gpu_name, gpu_vram_mb);
                        }
                    }
                }
            }
        }
    }

    #[cfg(target_os = "linux")]
    {
        if let Ok(output) = Command::new("lspci")
            .args(["-v"])
            .output()
        {
            if let Ok(output_str) = str::from_utf8(&output.stdout) {
                for line in output_str.lines() {
                    if line.contains("VGA") || line.contains("3D") {
                        if let Some(name_start) = line.find(':') {
                            gpu_name = String::from(line[name_start + 1..].trim());
                            break;
                        }
                    }
                }
            }
        }
    }

    #[cfg(target_os = "macos")]
    {
        if let Ok(output) = Command::new("system_profiler")
            .args(["SPDisplaysDataType"])
            .output()
        {
            if let Ok(output_str) = str::from_utf8(&output.stdout) {
                for line in output_str.lines() {
                    if line.contains("Chipset Model:") {
                        if let Some(name_start) = line.find(':') {
                            gpu_name = String::from(line[name_start + 1..].trim());
                            break;
                        }
                    }
                    if line.contains("VRAM") {
                        if let Some(vram_start) = line.find(':') {
                            let vram_str = line[vram_start + 1..].trim();
                            if vram_str.contains("MB") {
                                if let Ok(vram) = vram_str.replace("MB", "").trim().parse::<u64>() {
                                    gpu_vram_mb = vram;
                                }
                            } else if vram_str.contains("GB") {
                                if let Ok(vram) = vram_str.replace("GB", "").trim().parse::<u64>() {
                                    gpu_vram_mb = vram * 1024;
                                }
                            }
                            break;
                        }
                    }
                }
            }
        }
    }

    for component in sys.components() {
        let label = component.label().to_lowercase();
        if label.contains("gpu") || label.contains("graphics") {
            gpu_name = String::from(component.label());
            break;
        }
    }

    let ram_gb = sys.total_memory() / 1024 / 1024;
    if ram_gb >= 32 {
        gpu_vram_mb = 8192;
    } else if ram_gb >= 16 {
        gpu_vram_mb = 6144;
    } else if ram_gb >= 8 {
        gpu_vram_mb = 4096;
    }

    (gpu_name, gpu_vram_mb)
}

pub fn meets_minimum_requirements(info: &SystemInfo) -> bool {
    let has_enough_cores = info.cpu_cores >= 4;
    let has_enough_ram = info.ram_gb >= 8;
    let has_enough_vram = info.gpu_vram_mb >= 6 * 1024;

    has_enough_cores && has_enough_ram && has_enough_vram
}

pub fn meets_recommended_requirements(info: &SystemInfo) -> bool {
    let has_enough_cores = info.cpu_cores >= 8;
    let has_enough_ram = info.ram_gb >= 16;
    let has_enough_vram = info.gpu_vram_mb >= 8 * 1024;

    has_enough_cores && has_enough_ram && has_enough_vram
}

pub fn get_performance_tier(info: &SystemInfo) -> &'static str {
    if meets_recommended_requirements(info) {
        "High"
    } else if meets_minimum_requirements(info) {
        "Medium"
    } else {
        "Low"
    }
}
