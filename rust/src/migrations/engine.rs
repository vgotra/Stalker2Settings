use rusqlite::{Connection, Result, params};
use crate::models::{Setting, SettingValueType};

pub fn migrate(conn: &mut Connection) -> Result<()> {
    // /script/engine.engine section
    let settings = vec![
        Setting {
            name: "bAllowMultiThreadedShaderCompile".to_string(),
            section: "/script/engine.engine".to_string(),
            description: "Enables multithreaded shader compilation for improved loading times on multi-core CPUs".to_string(),
            value_type: SettingValueType::Enum(vec!["True".to_string(), "False".to_string()]),
            current_value: "True".to_string(),
            default_value: "True".to_string(),
            min_value: None,
            max_value: None,
            impact: "Performance".to_string(),
        },
    ];

    // Insert settings into the database
    let tx = conn.transaction()?;
    
    for setting in settings {
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

        tx.execute(
            "INSERT OR REPLACE INTO settings 
             (name, section, description, value_type, default_value, min_value, max_value, impact, possible_values)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                setting.name,
                setting.section,
                setting.description,
                value_type_str,
                setting.default_value,
                setting.min_value,
                setting.max_value,
                setting.impact,
                possible_values,
            ],
        )?;
    }

    tx.commit()?;
    Ok(())
}