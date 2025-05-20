use rusqlite::{Connection, Result, params};
use std::path::Path;
use crate::models::{Setting, SettingsPreset};
use std::collections::HashMap;
use chrono;

/// Initialize the database connection and create tables if they don't exist
pub fn initialize_db() -> Result<Connection> {
    let db_path = Path::new("settings.db");
    let db_exists = db_path.exists();

    let mut conn = Connection::open(db_path)?;

    if !db_exists {
        // Create tables
        conn.execute(
            "CREATE TABLE presets (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                description TEXT,
                created_at TEXT NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE preset_settings (
                preset_id INTEGER,
                setting_name TEXT NOT NULL,
                setting_value TEXT NOT NULL,
                PRIMARY KEY (preset_id, setting_name),
                FOREIGN KEY (preset_id) REFERENCES presets(id) ON DELETE CASCADE
            )",
            [],
        )?;

        // Create default presets
        create_default_presets(&mut conn)?;
    }

    Ok(conn)
}

/// Create default presets in the database
fn create_default_presets(conn: &mut Connection) -> Result<()> {
    // Create a "Default" preset with default settings
    let default_preset = SettingsPreset {
        id: None,
        name: "Default".to_string(),
        description: "Default settings from Engine.ini".to_string(),
        created_at: chrono::Local::now().to_rfc3339(),
        settings: HashMap::new(), // Will be populated with default values
    };

    save_preset(conn, &default_preset)?;

    // Create presets for different performance levels
    let performance_presets = [
        ("Low", "Optimized for low-end systems"),
        ("Medium", "Balanced performance and quality"),
        ("High", "High quality for powerful systems"),
        ("Ultra", "Maximum quality for high-end systems"),
    ];

    for (name, desc) in performance_presets.iter() {
        let preset = SettingsPreset {
            id: None,
            name: name.to_string(),
            description: desc.to_string(),
            created_at: chrono::Local::now().to_rfc3339(),
            settings: HashMap::new(), // Will be populated with appropriate values
        };

        save_preset(conn, &preset)?;
    }

    Ok(())
}

/// Save a settings preset to the database
pub fn save_preset(conn: &mut Connection, preset: &SettingsPreset) -> Result<i64> {
    let tx = conn.transaction()?;

    // Insert or update preset
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

    // Delete existing settings for this preset
    tx.execute(
        "DELETE FROM preset_settings WHERE preset_id = ?1",
        params![preset_id],
    )?;

    // Insert new settings
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

/// Load a settings preset from the database by ID
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

    // Load settings for this preset
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

/// Get all presets from the database
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

/// Delete a preset from the database
pub fn delete_preset(conn: &mut Connection, id: i64) -> Result<()> {
    conn.execute("DELETE FROM presets WHERE id = ?1", params![id])?;
    Ok(())
}
