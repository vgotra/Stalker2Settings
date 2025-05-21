use rusqlite::{Connection, Result, params};
use crate::models::{Setting, SettingValueType};

pub fn migrate(conn: &mut Connection) -> Result<()> {
    // Core.System section
    let settings = vec![
        Setting {
            name: "r.XGEShaderCompile".to_string(),
            section: "Core.System".to_string(),
            description: "Enables XGE shader compilation for distributed shader compilation".to_string(),
            value_type: SettingValueType::Boolean,
            current_value: "1".to_string(),
            default_value: "1".to_string(),
            min_value: None,
            max_value: None,
            impact: "Performance".to_string(),
        },
        Setting {
            name: "r.XGEShaderCompile.Mode".to_string(),
            section: "Core.System".to_string(),
            description: "Mode for XGE shader compilation (1=Local, 2=Distributed, 3=Auto)".to_string(),
            value_type: SettingValueType::Integer,
            current_value: "3".to_string(),
            default_value: "3".to_string(),
            min_value: Some("1".to_string()),
            max_value: Some("3".to_string()),
            impact: "Performance".to_string(),
        },
        Setting {
            name: "r.IoDispatcher.CacheSize".to_string(),
            section: "Core.System".to_string(),
            description: "Size of IO dispatcher cache in MB".to_string(),
            value_type: SettingValueType::Integer,
            current_value: "4096".to_string(),
            default_value: "4096".to_string(),
            min_value: Some("1024".to_string()),
            max_value: Some("8192".to_string()),
            impact: "Performance".to_string(),
        },
        Setting {
            name: "r.IoDispatcher.AsyncBatchReadSizeKB".to_string(),
            section: "Core.System".to_string(),
            description: "Size of asynchronous batch reads in KB".to_string(),
            value_type: SettingValueType::Integer,
            current_value: "4096".to_string(),
            default_value: "4096".to_string(),
            min_value: Some("1024".to_string()),
            max_value: Some("8192".to_string()),
            impact: "Performance".to_string(),
        },
        Setting {
            name: "+Suppress".to_string(),
            section: "Core.System".to_string(),
            description: "Suppresses specific log categories (Scriptwarning, Error, Scriptlog, Warning)".to_string(),
            value_type: SettingValueType::String,
            current_value: "Scriptwarning".to_string(),
            default_value: "Scriptwarning".to_string(),
            min_value: None,
            max_value: None,
            impact: "Debugging".to_string(),
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