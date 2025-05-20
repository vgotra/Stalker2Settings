use rusqlite::{Connection, Result, params};
use crate::models::{Setting, SettingValueType};

// This function has been replaced by individual migration files

// Import all migration modules
mod system_settings;
mod rendering_thread;
mod error_handling;
mod streaming_settings;
mod renderer_settings;
mod input_settings;
mod core_system;
mod texture_streaming;
mod engine;
mod shader_compiler;
mod core_log;
mod garbage_collection;

pub fn run_migrations(conn: &mut Connection) -> Result<()> {
    create_settings_table(conn)?;

    // Run all migrations
    system_settings::migrate(conn)?;
    rendering_thread::migrate(conn)?;
    error_handling::migrate(conn)?;
    streaming_settings::migrate(conn)?;
    renderer_settings::migrate(conn)?;
    input_settings::migrate(conn)?;
    core_system::migrate(conn)?;
    texture_streaming::migrate(conn)?;
    engine::migrate(conn)?;
    shader_compiler::migrate(conn)?;
    core_log::migrate(conn)?;
    garbage_collection::migrate(conn)?;

    Ok(())
}

fn create_settings_table(conn: &mut Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS settings (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            section TEXT NOT NULL,
            description TEXT NOT NULL,
            value_type TEXT NOT NULL,
            default_value TEXT NOT NULL,
            min_value TEXT,
            max_value TEXT,
            impact TEXT NOT NULL,
            possible_values TEXT,
            UNIQUE(section, name)
        )",
        [],
    )?;

    Ok(())
}

// This function has been replaced by individual migration files

pub fn get_all_settings(conn: &Connection) -> Result<Vec<Setting>> {
    let mut stmt = conn.prepare(
        "SELECT name, section, description, value_type, default_value, min_value, max_value, impact, possible_values
         FROM settings
         ORDER BY section, name"
    )?;

    let setting_iter = stmt.query_map([], |row| {
        let name: String = row.get(0)?;
        let section: String = row.get(1)?;
        let description: String = row.get(2)?;
        let value_type_str: String = row.get(3)?;
        let default_value: String = row.get(4)?;
        let min_value: Option<String> = row.get(5)?;
        let max_value: Option<String> = row.get(6)?;
        let impact: String = row.get(7)?;
        let possible_values: Option<String> = row.get(8)?;

        let value_type = match value_type_str.as_str() {
            "Boolean" => SettingValueType::Boolean,
            "Integer" => SettingValueType::Integer,
            "Float" => SettingValueType::Float,
            "Enum" => {
                if let Some(values_json) = possible_values {
                    let values: Vec<String> = serde_json::from_str(&values_json).unwrap_or_default();
                    SettingValueType::Enum(values)
                } else {
                    SettingValueType::String
                }
            },
            _ => SettingValueType::String,
        };

        Ok(Setting {
            name,
            section,
            description,
            value_type,
            current_value: default_value.clone(),
            default_value,
            min_value,
            max_value,
            impact,
        })
    })?;

    let mut settings = Vec::new();
    for setting_result in setting_iter {
        settings.push(setting_result?);
    }

    Ok(settings)
}

pub fn save_setting(conn: &mut Connection, setting: &Setting) -> Result<()> {
    let possible_values = match &setting.value_type {
        SettingValueType::Enum(values) => Some(serde_json::to_string(values).unwrap_or_default()),
        _ => None,
    };

    let value_type_str = match setting.value_type {
        SettingValueType::Boolean => "Boolean",
        SettingValueType::Integer => "Integer",
        SettingValueType::Float => "Float",
        SettingValueType::String => "String",
        SettingValueType::Enum(_) => "Enum",
    };

    conn.execute(
        "UPDATE settings 
         SET description = ?3, value_type = ?4, default_value = ?5, 
             min_value = ?6, max_value = ?7, impact = ?8, possible_values = ?9
         WHERE section = ?1 AND name = ?2",
        params![
            setting.section,
            setting.name,
            setting.description,
            value_type_str,
            setting.default_value,
            setting.min_value,
            setting.max_value,
            setting.impact,
            possible_values,
        ],
    )?;

    Ok(())
}
