use std::collections::HashMap;
use std::fs::File;
use std::io::{self, Write};
use std::path::Path;
use configparser::ini::Ini;
use crate::models::{Setting, SettingsPreset};
use crate::db;
use rusqlite::Connection;

pub fn parse_ini_file(file_path: &Path) -> io::Result<HashMap<String, HashMap<String, String>>> {
    let mut ini = Ini::new();
    let _ = ini.load(file_path.to_str().unwrap());
    let mut result = HashMap::new();
    let ini_map = ini.get_map_ref();

    for (section_name, properties) in ini_map {
        let mut section_map = HashMap::new();
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


pub fn generate_engine_ini_from_preset(preset: &SettingsPreset) -> io::Result<()> {
    let mut ini_data = HashMap::new();

    for (key, value) in &preset.settings {
        let parts: Vec<&str> = key.split('.').collect();
        if parts.len() == 2 {
            let section = parts[0];
            let name = parts[1];
            let section_map = ini_data.entry(String::from(section)).or_insert_with(HashMap::new);
            section_map.insert(String::from(name), value.clone());
        }
    }

    generate_ini_file(Path::new("../Engine.ini"), &ini_data)
}


pub fn generate_recommended_settings(system_info: &crate::models::SystemInfo, conn: Option<&Connection>) -> SettingsPreset {
    // Get the performance tier based on system info
    let tier = crate::system::get_performance_tier(system_info);

    // Function to find and return a preset matching the tier
    let find_preset = |connection: &Connection| -> Option<SettingsPreset> {
        if let Ok(presets) = db::get_all_presets(connection) {
            for preset in presets {
                if preset.name == tier {
                    // Found a matching preset, return it with updated description
                    return Some(SettingsPreset {
                        id: preset.id,
                        name: String::from("Recommended"),
                        description: format!("Recommended settings for your system: {} with {} VRAM", 
                                           system_info.gpu_name, system_info.gpu_vram_mb),
                        created_at: chrono::Local::now().to_rfc3339(),
                        settings: preset.settings,
                    });
                }
            }
        }
        None
    };

    // Try to get preset from provided connection or create a temporary one
    if let Some(connection) = conn {
        if let Some(preset) = find_preset(connection) {
            return preset;
        }
    } else if let Ok(temp_conn) = Connection::open(Path::new("../settings.db")) {
        if let Some(preset) = find_preset(&temp_conn) {
            return preset;
        }
    }

    // Fallback to hardcoded settings if no preset found
    let mut settings = HashMap::new();

    settings.insert(String::from("SystemSettings.r.VSync"), String::from("0"));
    settings.insert(String::from("RenderingThread.bAllowThreadedRendering"), String::from("True"));
    settings.insert(String::from("RenderingThread.bAllowAsyncRenderThreadUpdates"), String::from("True"));
    settings.insert(String::from("Engine.InputSettings.RawMouseInputEnabled"), String::from("1"));
    settings.insert(String::from("Engine.InputSettings.bEnableMouseSmoothing"), String::from("False"));

    let vram_mb = system_info.gpu_vram_mb;
    let streaming_pool_size = if vram_mb > 8000 {
        12288
    } else if vram_mb > 6000 {
        8192
    } else if vram_mb > 4000 {
        4096
    } else {
        2048
    };

    settings.insert(
        String::from("SystemSettings.r.Streaming.PoolSize"),
        format!("{}", streaming_pool_size),
    );

    let cpu_cores = system_info.cpu_cores;
    if cpu_cores >= 8 {
        settings.insert(String::from("SystemSettings.r.AllowMultiThreadedShaderCreation"), String::from("1"));
        settings.insert(String::from("ShaderCompiler.bAllowCompilingThroughWorkerThreads"), String::from("True"));
        settings.insert(String::from("ShaderCompiler.NumUnusedShaderCompilingThreads"), String::from("3"));
    } else if cpu_cores >= 4 {
        settings.insert(String::from("SystemSettings.r.AllowMultiThreadedShaderCreation"), String::from("1"));
        settings.insert(String::from("ShaderCompiler.bAllowCompilingThroughWorkerThreads"), String::from("True"));
        settings.insert(String::from("ShaderCompiler.NumUnusedShaderCompilingThreads"), String::from("1"));
    } else {
        settings.insert(String::from("SystemSettings.r.AllowMultiThreadedShaderCreation"), String::from("0"));
        settings.insert(String::from("ShaderCompiler.bAllowCompilingThroughWorkerThreads"), String::from("False"));
    }

    SettingsPreset {
        id: None,
        name: String::from("Recommended"),
        description: format!("Recommended settings for your system: {} with {} VRAM", 
                            system_info.gpu_name, system_info.gpu_vram_mb),
        created_at: chrono::Local::now().to_rfc3339(),
        settings,
    }
}

pub fn load_settings_from_db(conn: &Connection) -> Result<Vec<Setting>, rusqlite::Error> {
    db::get_all_settings(conn)
}

pub fn save_setting_to_db(conn: &mut Connection, setting: &Setting) -> Result<(), rusqlite::Error> {
    db::save_setting(conn, setting)
}
