use rusqlite::{Connection, Result, params};
use crate::models::{Setting, SettingValueType};

pub fn migrate(conn: &mut Connection) -> Result<()> {
    // /script/engine.streamingsettings section
    let settings = vec![
        Setting {
            name: "s.AsyncLoadingThreadEnabled".to_string(),
            section: "/script/engine.streamingsettings".to_string(),
            description: "Enables asynchronous loading thread for improved streaming performance".to_string(),
            value_type: SettingValueType::Enum(vec!["True".to_string(), "False".to_string()]),
            current_value: "True".to_string(),
            default_value: "True".to_string(),
            min_value: None,
            max_value: None,
            impact: "Performance".to_string(),
        },
        Setting {
            name: "s.AsyncLoadingUseFullTimeLimit".to_string(),
            section: "/script/engine.streamingsettings".to_string(),
            description: "Uses full time limit for asynchronous loading".to_string(),
            value_type: SettingValueType::Boolean,
            current_value: "1".to_string(),
            default_value: "1".to_string(),
            min_value: None,
            max_value: None,
            impact: "Performance".to_string(),
        },
        Setting {
            name: "s.MinBulkDataSizeForAsyncLoading".to_string(),
            section: "/script/engine.streamingsettings".to_string(),
            description: "Minimum size of bulk data for asynchronous loading".to_string(),
            value_type: SettingValueType::Integer,
            current_value: "0".to_string(),
            default_value: "0".to_string(),
            min_value: Some("0".to_string()),
            max_value: Some("1024".to_string()),
            impact: "Performance".to_string(),
        },
        Setting {
            name: "s.PriorityAsyncLoadingExtraTime".to_string(),
            section: "/script/engine.streamingsettings".to_string(),
            description: "Extra time for priority asynchronous loading".to_string(),
            value_type: SettingValueType::Float,
            current_value: "0".to_string(),
            default_value: "0".to_string(),
            min_value: Some("0.0".to_string()),
            max_value: Some("10.0".to_string()),
            impact: "Performance".to_string(),
        },
        Setting {
            name: "s.AsyncLoadingTimeLimit".to_string(),
            section: "/script/engine.streamingsettings".to_string(),
            description: "Time limit for asynchronous loading in milliseconds".to_string(),
            value_type: SettingValueType::Integer,
            current_value: "4".to_string(),
            default_value: "4".to_string(),
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