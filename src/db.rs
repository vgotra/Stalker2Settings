use rusqlite::{Connection, Result, params};
use std::path::Path;
use crate::models::{Setting, SettingsPreset};
use std::collections::HashMap;
use chrono;
use crate::migrations;

pub fn initialize_db(recreate_db: bool) -> Result<Connection> {
    let db_path = Path::new("settings.db");
    let mut db_exists = db_path.exists();

    // If recreate_db is true and the database exists, delete it
    if recreate_db && db_exists {
        if let Err(e) = std::fs::remove_file(db_path) {
            return Err(rusqlite::Error::ToSqlConversionFailure(Box::new(e)));
        }
        println!("Recreating database...");
        db_exists = false;
    }

    let mut conn = Connection::open(db_path)?;

    if !db_exists || recreate_db {
        conn.execute(
            "CREATE TABLE IF NOT EXISTS presets (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                description TEXT,
                created_at TEXT NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS preset_settings (
                preset_id INTEGER,
                setting_name TEXT NOT NULL,
                setting_value TEXT NOT NULL,
                PRIMARY KEY (preset_id, setting_name),
                FOREIGN KEY (preset_id) REFERENCES presets(id) ON DELETE CASCADE
            )",
            [],
        )?;

        create_default_presets(&mut conn)?;
    }

    migrations::run_migrations(&mut conn)?;

    Ok(conn)
}

fn create_default_presets(conn: &mut Connection) -> Result<()> {
    // Parse Engine.ini to get default settings
    let engine_ini_path = std::path::Path::new("Engine.ini");
    let default_settings = match crate::config::parse_ini_file(engine_ini_path) {
        Ok(settings) => {
            let mut settings_map = HashMap::new();
            for (section, properties) in settings {
                for (key, value) in properties {
                    let setting_key = format!("{}.{}", section, key);
                    settings_map.insert(setting_key, value);
                }
            }
            settings_map
        },
        Err(_) => HashMap::new(),
    };

    // Create Default preset with settings from Engine.ini
    let default_preset = SettingsPreset {
        id: None,
        name: "Default".to_string(),
        description: "Default settings from Engine.ini".to_string(),
        created_at: chrono::Local::now().to_rfc3339(),
        settings: default_settings.clone(),
    };

    save_preset(conn, &default_preset)?;

    // Create performance presets with different settings
    let performance_presets = [
        ("Low", "Optimized for low-end systems"),
        ("Medium", "Balanced performance and quality"),
        ("High", "High quality for powerful systems"),
        ("Ultra", "Maximum quality for high-end systems"),
    ];

    for (name, desc) in performance_presets.iter() {
        let mut preset_settings = default_settings.clone();

        // Adjust settings based on performance tier
        match *name {
            "Low" => {
                // Low-end settings: prioritize performance over quality
                preset_settings.insert("SystemSettings.r.Streaming.PoolSize".to_string(), "2048".to_string());
                preset_settings.insert("SystemSettings.r.AllowMultiThreadedShaderCreation".to_string(), "1".to_string());
                preset_settings.insert("SystemSettings.r.TemporalAACurrentFrameWeight".to_string(), "0.1".to_string());
                preset_settings.insert("SystemSettings.r.TemporalAASamples".to_string(), "4".to_string());
                preset_settings.insert("SystemSettings.r.TemporalAASharpness".to_string(), "0.7".to_string());
                preset_settings.insert("SystemSettings.r.Tonemapper.Sharpen".to_string(), "0.5".to_string());
                preset_settings.insert("SystemSettings.r.RayTracing".to_string(), "0".to_string());
                preset_settings.insert("SystemSettings.r.Lumen".to_string(), "0".to_string());
                preset_settings.insert("SystemSettings.r.SSR".to_string(), "0".to_string());
                preset_settings.insert("SystemSettings.r.AmbientOcclusionLevels".to_string(), "0".to_string());
                preset_settings.insert("SystemSettings.r.ShadowQuality".to_string(), "1".to_string());
                preset_settings.insert("SystemSettings.r.VolumetricFog".to_string(), "0".to_string());
                preset_settings.insert("SystemSettings.r.MotionBlurQuality".to_string(), "0".to_string());
                preset_settings.insert("SystemSettings.r.DepthOfFieldQuality".to_string(), "0".to_string());
                preset_settings.insert("SystemSettings.r.BloomQuality".to_string(), "0".to_string());
            },
            "Medium" => {
                // Medium settings: balanced performance and quality
                preset_settings.insert("SystemSettings.r.Streaming.PoolSize".to_string(), "4096".to_string());
                preset_settings.insert("SystemSettings.r.AllowMultiThreadedShaderCreation".to_string(), "1".to_string());
                preset_settings.insert("SystemSettings.r.TemporalAACurrentFrameWeight".to_string(), "0.12".to_string());
                preset_settings.insert("SystemSettings.r.TemporalAASamples".to_string(), "6".to_string());
                preset_settings.insert("SystemSettings.r.TemporalAASharpness".to_string(), "0.8".to_string());
                preset_settings.insert("SystemSettings.r.Tonemapper.Sharpen".to_string(), "0.6".to_string());
                preset_settings.insert("SystemSettings.r.RayTracing".to_string(), "0".to_string());
                preset_settings.insert("SystemSettings.r.Lumen".to_string(), "0".to_string());
                preset_settings.insert("SystemSettings.r.SSR".to_string(), "0".to_string());
                preset_settings.insert("SystemSettings.r.AmbientOcclusionLevels".to_string(), "1".to_string());
                preset_settings.insert("SystemSettings.r.ShadowQuality".to_string(), "2".to_string());
                preset_settings.insert("SystemSettings.r.VolumetricFog".to_string(), "0".to_string());
                preset_settings.insert("SystemSettings.r.MotionBlurQuality".to_string(), "0".to_string());
                preset_settings.insert("SystemSettings.r.DepthOfFieldQuality".to_string(), "1".to_string());
                preset_settings.insert("SystemSettings.r.BloomQuality".to_string(), "1".to_string());
            },
            "High" => {
                // High settings: prioritize quality with good performance
                preset_settings.insert("SystemSettings.r.Streaming.PoolSize".to_string(), "8192".to_string());
                preset_settings.insert("SystemSettings.r.AllowMultiThreadedShaderCreation".to_string(), "1".to_string());
                preset_settings.insert("SystemSettings.r.TemporalAACurrentFrameWeight".to_string(), "0.15".to_string());
                preset_settings.insert("SystemSettings.r.TemporalAASamples".to_string(), "8".to_string());
                preset_settings.insert("SystemSettings.r.TemporalAASharpness".to_string(), "0.9".to_string());
                preset_settings.insert("SystemSettings.r.Tonemapper.Sharpen".to_string(), "0.7".to_string());
                preset_settings.insert("SystemSettings.r.RayTracing".to_string(), "0".to_string());
                preset_settings.insert("SystemSettings.r.Lumen".to_string(), "0".to_string());
                preset_settings.insert("SystemSettings.r.SSR".to_string(), "1".to_string());
                preset_settings.insert("SystemSettings.r.AmbientOcclusionLevels".to_string(), "2".to_string());
                preset_settings.insert("SystemSettings.r.ShadowQuality".to_string(), "3".to_string());
                preset_settings.insert("SystemSettings.r.VolumetricFog".to_string(), "1".to_string());
                preset_settings.insert("SystemSettings.r.MotionBlurQuality".to_string(), "1".to_string());
                preset_settings.insert("SystemSettings.r.DepthOfFieldQuality".to_string(), "2".to_string());
                preset_settings.insert("SystemSettings.r.BloomQuality".to_string(), "2".to_string());
            },
            "Ultra" => {
                // Ultra settings: maximum quality
                preset_settings.insert("SystemSettings.r.Streaming.PoolSize".to_string(), "12288".to_string());
                preset_settings.insert("SystemSettings.r.AllowMultiThreadedShaderCreation".to_string(), "1".to_string());
                preset_settings.insert("SystemSettings.r.TemporalAACurrentFrameWeight".to_string(), "0.15".to_string());
                preset_settings.insert("SystemSettings.r.TemporalAASamples".to_string(), "8".to_string());
                preset_settings.insert("SystemSettings.r.TemporalAASharpness".to_string(), "0.9".to_string());
                preset_settings.insert("SystemSettings.r.Tonemapper.Sharpen".to_string(), "0.8".to_string());
                preset_settings.insert("SystemSettings.r.RayTracing".to_string(), "1".to_string());
                preset_settings.insert("SystemSettings.r.RayTracing.Reflections".to_string(), "1".to_string());
                preset_settings.insert("SystemSettings.r.RayTracing.AmbientOcclusion".to_string(), "1".to_string());
                preset_settings.insert("SystemSettings.r.Lumen".to_string(), "1".to_string());
                preset_settings.insert("SystemSettings.r.Lumen.Reflections".to_string(), "1".to_string());
                preset_settings.insert("SystemSettings.r.SSR".to_string(), "1".to_string());
                preset_settings.insert("SystemSettings.r.AmbientOcclusionLevels".to_string(), "3".to_string());
                preset_settings.insert("SystemSettings.r.ShadowQuality".to_string(), "5".to_string());
                preset_settings.insert("SystemSettings.r.VolumetricFog".to_string(), "1".to_string());
                preset_settings.insert("SystemSettings.r.MotionBlurQuality".to_string(), "2".to_string());
                preset_settings.insert("SystemSettings.r.DepthOfFieldQuality".to_string(), "3".to_string());
                preset_settings.insert("SystemSettings.r.BloomQuality".to_string(), "3".to_string());
            },
            _ => {}
        }

        // Common settings for all presets
        preset_settings.insert("SystemSettings.r.VSync".to_string(), "0".to_string());
        preset_settings.insert("RenderingThread.bAllowThreadedRendering".to_string(), "True".to_string());
        preset_settings.insert("RenderingThread.bAllowAsyncRenderThreadUpdates".to_string(), "True".to_string());
        preset_settings.insert("Engine.InputSettings.RawMouseInputEnabled".to_string(), "1".to_string());
        preset_settings.insert("Engine.InputSettings.bEnableMouseSmoothing".to_string(), "False".to_string());

        let preset = SettingsPreset {
            id: None,
            name: name.to_string(),
            description: desc.to_string(),
            created_at: chrono::Local::now().to_rfc3339(),
            settings: preset_settings,
        };

        save_preset(conn, &preset)?;
    }

    Ok(())
}

pub fn save_preset(conn: &mut Connection, preset: &SettingsPreset) -> Result<i64> {
    let tx = conn.transaction()?;

    let preset_id = match preset.id {
        Some(id) => {
            tx.execute(
                "UPDATE presets SET name = ?1, description = ?2 WHERE id = ?3",
                params![preset.name, preset.description, id],
            )?;
            id
        },
        None => {
            tx.execute(
                "INSERT INTO presets (name, description, created_at) VALUES (?1, ?2, ?3)",
                params![preset.name, preset.description, preset.created_at],
            )?;
            tx.last_insert_rowid()
        }
    };

    tx.execute(
        "DELETE FROM preset_settings WHERE preset_id = ?1",
        params![preset_id],
    )?;

    for (name, value) in &preset.settings {
        tx.execute(
            "INSERT INTO preset_settings (preset_id, setting_name, setting_value) 
             VALUES (?1, ?2, ?3)",
            params![preset_id, name, value],
        )?;
    }

    tx.commit()?;

    Ok(preset_id)
}

pub fn load_preset_by_id(conn: &Connection, id: i64) -> Result<SettingsPreset> {
    let mut stmt = conn.prepare(
        "SELECT id, name, description, created_at FROM presets WHERE id = ?1"
    )?;

    let preset = stmt.query_row(params![id], |row| {
        let id = row.get(0)?;
        let name = row.get(1)?;
        let description = row.get(2)?;
        let created_at = row.get(3)?;

        Ok(SettingsPreset {
            id: Some(id),
            name,
            description,
            created_at,
            settings: HashMap::new(),
        })
    })?;

    let mut settings = HashMap::new();
    let mut stmt = conn.prepare(
        "SELECT setting_name, setting_value FROM preset_settings WHERE preset_id = ?1"
    )?;

    let rows = stmt.query_map(params![id], |row| {
        let name: String = row.get(0)?;
        let value: String = row.get(1)?;
        Ok((name, value))
    })?;

    for row in rows {
        let (name, value) = row?;
        settings.insert(name, value);
    }

    Ok(SettingsPreset {
        settings,
        ..preset
    })
}

pub fn get_all_presets(conn: &Connection) -> Result<Vec<SettingsPreset>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, description, created_at FROM presets ORDER BY name"
    )?;

    let preset_iter = stmt.query_map([], |row| {
        let id = row.get(0)?;
        let name = row.get(1)?;
        let description = row.get(2)?;
        let created_at = row.get(3)?;

        Ok(SettingsPreset {
            id: Some(id),
            name,
            description,
            created_at,
            settings: HashMap::new(),
        })
    })?;

    let mut presets = Vec::new();
    for preset_result in preset_iter {
        let preset = preset_result?;
        presets.push(load_preset_by_id(conn, preset.id.unwrap())?);
    }

    Ok(presets)
}

pub fn delete_preset(conn: &mut Connection, id: i64) -> Result<()> {
    conn.execute("DELETE FROM presets WHERE id = ?1", params![id])?;
    Ok(())
}

pub fn get_all_settings(conn: &Connection) -> Result<Vec<Setting>> {
    migrations::get_all_settings(conn)
}

pub fn save_setting(conn: &mut Connection, setting: &Setting) -> Result<()> {
    migrations::save_setting(conn, setting)
}
