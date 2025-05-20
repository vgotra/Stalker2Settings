use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::Path;
use configparser::ini::Ini;
use crate::models::{Setting, SettingValueType, SettingsPreset};

/// Parse an INI file into a HashMap of section -> (key -> value)
pub fn parse_ini_file(file_path: &Path) -> io::Result<HashMap<String, HashMap<String, String>>> {
    let mut ini = Ini::new();
    let _ = ini.load(file_path.to_str().unwrap());

    let mut result = HashMap::new();

    // Get all sections from the map
    let ini_map = ini.get_map_ref();

    for (section_name, properties) in ini_map {
        let mut section_map = HashMap::new();

        // Process properties for this section
        for (key, value_opt) in properties {
            if let Some(value) = value_opt {
                section_map.insert(key.clone(), value.clone());
            }
        }

        if !section_map.is_empty() {
            result.insert(section_name.clone(), section_map);
        }
    }

    Ok(result)
}

/// Generate an INI file from a HashMap of section -> (key -> value)
pub fn generate_ini_file(
    file_path: &Path,
    settings: &HashMap<String, HashMap<String, String>>,
) -> io::Result<()> {
    let mut ini_content = String::new();

    for (section, properties) in settings {
        ini_content.push_str(&format!("[{}]\n", section));

        for (key, value) in properties {
            ini_content.push_str(&format!("{}={}\n", key, value));
        }

        ini_content.push('\n');
    }

    let mut file = File::create(file_path)?;
    file.write_all(ini_content.as_bytes())?;

    Ok(())
}

/// Load all settings from Engine.ini with descriptions and metadata
pub fn load_engine_settings() -> io::Result<Vec<Setting>> {
    let engine_ini_path = Path::new("Engine.ini");
    let ini_data = parse_ini_file(engine_ini_path)?;

    let mut settings = Vec::new();

    // Add settings with descriptions
    for (section, properties) in &ini_data {
        for (key, value) in properties {
            let setting = create_setting_with_description(section, key, value);
            settings.push(setting);
        }
    }

    Ok(settings)
}

/// Create a Setting with description based on known settings
fn create_setting_with_description(section: &str, name: &str, value: &str) -> Setting {
    // Default values
    let mut description = "".to_string();
    let mut value_type = SettingValueType::String;
    let mut min_value = None;
    let mut max_value = None;
    let mut impact = "Performance".to_string();

    // Define known settings with descriptions
    match (section, name) {
        ("SystemSettings", "r.PSOWarmup.WarmupMaterials") => {
            description = "Enable shader precompilation for materials".to_string();
            value_type = SettingValueType::Boolean;
            impact = "Loading Time".to_string();
        },
        ("SystemSettings", "r.VSync") => {
            description = "Enable vertical sync to prevent screen tearing".to_string();
            value_type = SettingValueType::Boolean;
            impact = "Performance".to_string();
        },
        ("SystemSettings", "r.Streaming.PoolSize") => {
            description = "Size of texture streaming pool in MB".to_string();
            value_type = SettingValueType::Integer;
            min_value = Some("1024".to_string());
            max_value = Some("16384".to_string());
            impact = "Visual Quality".to_string();
        },
        ("SystemSettings", "r.TemporalAACurrentFrameWeight") => {
            description = "Weight of current frame in temporal AA".to_string();
            value_type = SettingValueType::Float;
            min_value = Some("0.0".to_string());
            max_value = Some("1.0".to_string());
            impact = "Visual Quality".to_string();
        },
        ("SystemSettings", "r.TemporalAASamples") => {
            description = "Number of temporal AA samples".to_string();
            value_type = SettingValueType::Integer;
            min_value = Some("4".to_string());
            max_value = Some("64".to_string());
            impact = "Visual Quality".to_string();
        },
        ("SystemSettings", "r.TemporalAASharpness") => {
            description = "Sharpness of temporal AA".to_string();
            value_type = SettingValueType::Float;
            min_value = Some("0.0".to_string());
            max_value = Some("1.0".to_string());
            impact = "Visual Quality".to_string();
        },
        ("SystemSettings", "r.Tonemapper.Sharpen") => {
            description = "Sharpness of the tonemapper".to_string();
            value_type = SettingValueType::Float;
            min_value = Some("0.0".to_string());
            max_value = Some("1.0".to_string());
            impact = "Visual Quality".to_string();
        },
        ("SystemSettings", "r.RHICmdBypass") => {
            description = "Bypass RHI command list".to_string();
            value_type = SettingValueType::Boolean;
            impact = "Performance".to_string();
        },
        ("SystemSettings", "r.AllowMultiThreadedShaderCreation") => {
            description = "Allow multithreaded shader creation".to_string();
            value_type = SettingValueType::Boolean;
            impact = "Performance".to_string();
        },
        ("RenderingThread", "bAllowThreadedRendering") => {
            description = "Allow threaded rendering".to_string();
            value_type = SettingValueType::Enum(vec!["True".to_string(), "False".to_string()]);
            impact = "Performance".to_string();
        },
        ("RenderingThread", "bAllowAsyncRenderThreadUpdates") => {
            description = "Allow async render thread updates".to_string();
            value_type = SettingValueType::Enum(vec!["True".to_string(), "False".to_string()]);
            impact = "Performance".to_string();
        },
        ("Engine.InputSettings", "RawMouseInputEnabled") => {
            description = "Enable raw mouse input".to_string();
            value_type = SettingValueType::Integer;
            impact = "Input".to_string();
        },
        ("Engine.InputSettings", "bEnableMouseSmoothing") => {
            description = "Enable mouse smoothing".to_string();
            value_type = SettingValueType::Enum(vec!["True".to_string(), "False".to_string()]);
            impact = "Input".to_string();
        },
        // Add more settings as needed
        _ => {
            // For unknown settings, try to infer the type
            if value == "0" || value == "1" || value == "True" || value == "False" {
                value_type = SettingValueType::Boolean;
            } else if value.parse::<i64>().is_ok() {
                value_type = SettingValueType::Integer;
            } else if value.parse::<f64>().is_ok() {
                value_type = SettingValueType::Float;
            }

            description = format!("Setting for {}", name);
        }
    }

    Setting {
        name: name.to_string(),
        section: section.to_string(),
        description,
        value_type,
        current_value: value.to_string(),
        default_value: value.to_string(),
        min_value,
        max_value,
        impact,
    }
}

/// Generate Engine.ini file from a settings preset
pub fn generate_engine_ini_from_preset(preset: &SettingsPreset) -> io::Result<()> {
    let mut ini_data = HashMap::new();

    // Group settings by section
    for (key, value) in &preset.settings {
        // Parse the key to get section and setting name
        // Format is expected to be "section.name"
        let parts: Vec<&str> = key.split('.').collect();
        if parts.len() == 2 {
            let section = parts[0];
            let name = parts[1];

            let section_map = ini_data.entry(section.to_string()).or_insert_with(HashMap::new);
            section_map.insert(name.to_string(), value.clone());
        }
    }

    // Generate the INI file
    generate_ini_file(Path::new("Engine.ini"), &ini_data)
}

/// Create a preset from the current Engine.ini file
pub fn create_preset_from_engine_ini(name: &str, description: &str) -> io::Result<SettingsPreset> {
    let engine_ini_path = Path::new("Engine.ini");
    let ini_data = parse_ini_file(engine_ini_path)?;

    let mut settings = HashMap::new();

    // Flatten the settings into a single map with keys in the format "section.name"
    for (section, properties) in ini_data {
        for (key, value) in properties {
            let setting_key = format!("{}.{}", section, key);
            settings.insert(setting_key, value);
        }
    }

    Ok(SettingsPreset {
        id: None,
        name: name.to_string(),
        description: description.to_string(),
        created_at: chrono::Local::now().to_rfc3339(),
        settings,
    })
}

/// Generate recommended settings based on system info
pub fn generate_recommended_settings(
    system_info: &crate::models::SystemInfo,
) -> SettingsPreset {
    let mut settings = HashMap::new();

    // Base settings for all configurations
    settings.insert("SystemSettings.r.VSync".to_string(), "0".to_string());
    settings.insert("RenderingThread.bAllowThreadedRendering".to_string(), "True".to_string());
    settings.insert("RenderingThread.bAllowAsyncRenderThreadUpdates".to_string(), "True".to_string());
    settings.insert("Engine.InputSettings.RawMouseInputEnabled".to_string(), "1".to_string());
    settings.insert("Engine.InputSettings.bEnableMouseSmoothing".to_string(), "False".to_string());

    // Adjust settings based on GPU VRAM
    let vram_mb = system_info.gpu_vram_mb;
    let streaming_pool_size = if vram_mb > 8000 {
        12288 // 12GB for high-end GPUs
    } else if vram_mb > 6000 {
        8192 // 8GB for mid-range GPUs
    } else if vram_mb > 4000 {
        4096 // 4GB for lower-mid GPUs
    } else {
        2048 // 2GB for low-end GPUs
    };

    settings.insert(
        "SystemSettings.r.Streaming.PoolSize".to_string(),
        streaming_pool_size.to_string(),
    );

    // Adjust settings based on CPU cores
    let cpu_cores = system_info.cpu_cores;
    if cpu_cores >= 8 {
        settings.insert("SystemSettings.r.AllowMultiThreadedShaderCreation".to_string(), "1".to_string());
        settings.insert("ShaderCompiler.bAllowCompilingThroughWorkerThreads".to_string(), "True".to_string());
        settings.insert("ShaderCompiler.NumUnusedShaderCompilingThreads".to_string(), "3".to_string());
    } else if cpu_cores >= 4 {
        settings.insert("SystemSettings.r.AllowMultiThreadedShaderCreation".to_string(), "1".to_string());
        settings.insert("ShaderCompiler.bAllowCompilingThroughWorkerThreads".to_string(), "True".to_string());
        settings.insert("ShaderCompiler.NumUnusedShaderCompilingThreads".to_string(), "1".to_string());
    } else {
        settings.insert("SystemSettings.r.AllowMultiThreadedShaderCreation".to_string(), "0".to_string());
        settings.insert("ShaderCompiler.bAllowCompilingThroughWorkerThreads".to_string(), "False".to_string());
    }

    // Create the preset
    SettingsPreset {
        id: None,
        name: "Recommended".to_string(),
        description: format!("Recommended settings for your system: {} with {} VRAM", 
                            system_info.gpu_name, system_info.gpu_vram_mb),
        created_at: chrono::Local::now().to_rfc3339(),
        settings,
    }
}
