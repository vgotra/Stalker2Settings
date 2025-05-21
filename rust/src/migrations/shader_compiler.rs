use rusqlite::{Connection, Result, params};
use crate::models::{Setting, SettingValueType};

pub fn migrate(conn: &mut Connection) -> Result<()> {
    // ShaderCompiler section
    let settings = vec![
        Setting {
            name: "bAllowAsynchronousShaderCompiling".to_string(),
            section: "ShaderCompiler".to_string(),
            description: "Enables asynchronous shader compilation for improved loading times".to_string(),
            value_type: SettingValueType::Enum(vec!["True".to_string(), "False".to_string()]),
            current_value: "True".to_string(),
            default_value: "True".to_string(),
            min_value: None,
            max_value: None,
            impact: "Performance".to_string(),
        },
        Setting {
            name: "bAllowCompilingThroughWorkerThreads".to_string(),
            section: "ShaderCompiler".to_string(),
            description: "Enables shader compilation through worker threads for improved performance".to_string(),
            value_type: SettingValueType::Enum(vec!["True".to_string(), "False".to_string()]),
            current_value: "True".to_string(),
            default_value: "True".to_string(),
            min_value: None,
            max_value: None,
            impact: "Performance".to_string(),
        },
        Setting {
            name: "NumUnusedShaderCompilingThreads".to_string(),
            section: "ShaderCompiler".to_string(),
            description: "Number of unused shader compiling threads. Higher values improve compilation speed but use more CPU resources".to_string(),
            value_type: SettingValueType::Integer,
            current_value: "1".to_string(),
            default_value: "1".to_string(),
            min_value: Some("0".to_string()),
            max_value: Some("16".to_string()),
            impact: "Performance".to_string(),
        },
        Setting {
            name: "bAsyncShaderCompileWorkerThreads".to_string(),
            section: "ShaderCompiler".to_string(),
            description: "Enables asynchronous shader compilation worker threads".to_string(),
            value_type: SettingValueType::Enum(vec!["True".to_string(), "False".to_string()]),
            current_value: "True".to_string(),
            default_value: "True".to_string(),
            min_value: None,
            max_value: None,
            impact: "Performance".to_string(),
        },
        Setting {
            name: "bEnableOptimizedShaderCompilation".to_string(),
            section: "ShaderCompiler".to_string(),
            description: "Enables optimized shader compilation for improved performance".to_string(),
            value_type: SettingValueType::Enum(vec!["True".to_string(), "False".to_string()]),
            current_value: "True".to_string(),
            default_value: "True".to_string(),
            min_value: None,
            max_value: None,
            impact: "Performance".to_string(),
        },
        Setting {
            name: "MaxShaderJobBatchSize".to_string(),
            section: "ShaderCompiler".to_string(),
            description: "Maximum size of shader job batches. Higher values improve throughput but may cause longer individual compilation times".to_string(),
            value_type: SettingValueType::Integer,
            current_value: "50".to_string(),
            default_value: "50".to_string(),
            min_value: Some("10".to_string()),
            max_value: Some("100".to_string()),
            impact: "Performance".to_string(),
        },
        Setting {
            name: "MaxShaderJobs".to_string(),
            section: "ShaderCompiler".to_string(),
            description: "Maximum number of shader jobs. Higher values allow more shaders to be compiled in parallel but use more memory".to_string(),
            value_type: SettingValueType::Integer,
            current_value: "500".to_string(),
            default_value: "500".to_string(),
            min_value: Some("100".to_string()),
            max_value: Some("1000".to_string()),
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