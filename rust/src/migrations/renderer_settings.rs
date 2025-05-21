use rusqlite::{Connection, Result, params};
use crate::models::{Setting, SettingValueType};

pub fn migrate(conn: &mut Connection) -> Result<()> {
    // Engine.RendererSettings section
    let settings = vec![
        Setting {
            name: "r.AsyncCreateLightPrimitiveInteractions".to_string(),
            section: "Engine.RendererSettings".to_string(),
            description: "Enables asynchronous creation of light primitive interactions for improved performance".to_string(),
            value_type: SettingValueType::Boolean,
            current_value: "1".to_string(),
            default_value: "1".to_string(),
            min_value: None,
            max_value: None,
            impact: "Performance".to_string(),
        },
        Setting {
            name: "r.RDG.AsyncCompute".to_string(),
            section: "Engine.RendererSettings".to_string(),
            description: "Enables asynchronous compute for Render Dependency Graph".to_string(),
            value_type: SettingValueType::Boolean,
            current_value: "1".to_string(),
            default_value: "1".to_string(),
            min_value: None,
            max_value: None,
            impact: "Performance".to_string(),
        },
        Setting {
            name: "r.AsyncPipelineCompile".to_string(),
            section: "Engine.RendererSettings".to_string(),
            description: "Enables asynchronous pipeline compilation for improved loading times".to_string(),
            value_type: SettingValueType::Boolean,
            current_value: "1".to_string(),
            default_value: "1".to_string(),
            min_value: None,
            max_value: None,
            impact: "Performance".to_string(),
        },
        Setting {
            name: "r.AmbientOcclusion.AsyncComputeBudget".to_string(),
            section: "Engine.RendererSettings".to_string(),
            description: "Budget for asynchronous compute of ambient occlusion".to_string(),
            value_type: SettingValueType::Integer,
            current_value: "1".to_string(),
            default_value: "1".to_string(),
            min_value: Some("0".to_string()),
            max_value: Some("2".to_string()),
            impact: "Performance".to_string(),
        },
        Setting {
            name: "r.EnableAsyncComputeVolumetricFog".to_string(),
            section: "Engine.RendererSettings".to_string(),
            description: "Enables asynchronous compute for volumetric fog".to_string(),
            value_type: SettingValueType::Boolean,
            current_value: "1".to_string(),
            default_value: "1".to_string(),
            min_value: None,
            max_value: None,
            impact: "Performance".to_string(),
        },
        Setting {
            name: "r.Streaming.UseAsyncRequestsForDDC".to_string(),
            section: "Engine.RendererSettings".to_string(),
            description: "Enables asynchronous requests for Derived Data Cache".to_string(),
            value_type: SettingValueType::Boolean,
            current_value: "1".to_string(),
            default_value: "1".to_string(),
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