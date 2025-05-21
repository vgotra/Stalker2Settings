use rusqlite::{Connection, Result, params};
use crate::models::{Setting, SettingValueType};

pub fn migrate(conn: &mut Connection) -> Result<()> {
    // TextureStreaming section
    let settings = vec![
        Setting {
            name: "r.TextureStreaming".to_string(),
            section: "TextureStreaming".to_string(),
            description: "Enables texture streaming for improved memory usage".to_string(),
            value_type: SettingValueType::Boolean,
            current_value: "1".to_string(),
            default_value: "1".to_string(),
            min_value: None,
            max_value: None,
            impact: "Performance".to_string(),
        },
        Setting {
            name: "r.Streaming.FullyLoadUsedTextures".to_string(),
            section: "TextureStreaming".to_string(),
            description: "Fully loads used textures for improved visual quality".to_string(),
            value_type: SettingValueType::Boolean,
            current_value: "1".to_string(),
            default_value: "1".to_string(),
            min_value: None,
            max_value: None,
            impact: "Visual Quality".to_string(),
        },
        Setting {
            name: "r.Streaming.FullyLoadUsedTextures_Always".to_string(),
            section: "TextureStreaming".to_string(),
            description: "Always fully loads used textures, even during performance-critical moments".to_string(),
            value_type: SettingValueType::Boolean,
            current_value: "1".to_string(),
            default_value: "1".to_string(),
            min_value: None,
            max_value: None,
            impact: "Visual Quality".to_string(),
        },
        Setting {
            name: "r.Streaming.DefragDynamicBounds".to_string(),
            section: "TextureStreaming".to_string(),
            description: "Enables defragmentation of dynamic texture streaming bounds for improved memory usage".to_string(),
            value_type: SettingValueType::Boolean,
            current_value: "1".to_string(),
            default_value: "1".to_string(),
            min_value: None,
            max_value: None,
            impact: "Performance".to_string(),
        },
        Setting {
            name: "r.Streaming.FramesForFullUpdate".to_string(),
            section: "TextureStreaming".to_string(),
            description: "Number of frames between full texture streaming updates".to_string(),
            value_type: SettingValueType::Integer,
            current_value: "1".to_string(),
            default_value: "1".to_string(),
            min_value: Some("1".to_string()),
            max_value: Some("10".to_string()),
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