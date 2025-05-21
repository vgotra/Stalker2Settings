use rusqlite::{Connection, Result, params};
use crate::models::{Setting, SettingValueType};

pub fn migrate(conn: &mut Connection) -> Result<()> {
    // Engine.InputSettings section
    let settings = vec![
        Setting {
            name: "RawMouseInputEnabled".to_string(),
            section: "Engine.InputSettings".to_string(),
            description: "Enables raw mouse input for more precise cursor movement".to_string(),
            value_type: SettingValueType::Boolean,
            current_value: "1".to_string(),
            default_value: "1".to_string(),
            min_value: None,
            max_value: None,
            impact: "Input".to_string(),
        },
        Setting {
            name: "bEnableMouseSmoothing".to_string(),
            section: "Engine.InputSettings".to_string(),
            description: "Enables mouse smoothing. Disabling provides more responsive but potentially less smooth cursor movement".to_string(),
            value_type: SettingValueType::Enum(vec!["True".to_string(), "False".to_string()]),
            current_value: "False".to_string(),
            default_value: "False".to_string(),
            min_value: None,
            max_value: None,
            impact: "Input".to_string(),
        },
        Setting {
            name: "bViewAccelerationEnabled".to_string(),
            section: "Engine.InputSettings".to_string(),
            description: "Enables view acceleration for smoother camera movement".to_string(),
            value_type: SettingValueType::Enum(vec!["True".to_string(), "False".to_string()]),
            current_value: "False".to_string(),
            default_value: "False".to_string(),
            min_value: None,
            max_value: None,
            impact: "Input".to_string(),
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