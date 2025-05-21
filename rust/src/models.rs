use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents the type of a setting value
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SettingValueType {
    Boolean,
    Integer,
    Float,
    String,
    Enum(Vec<String>),
}

/// Represents a single setting with its metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Setting {
    pub name: String,
    pub section: String,
    pub description: String,
    pub value_type: SettingValueType,
    pub current_value: String,
    pub default_value: String,
    pub min_value: Option<String>,
    pub max_value: Option<String>,
    pub impact: String, // Performance, Visual, etc.
}

/// Represents a named collection of settings (a preset)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingsPreset {
    pub id: Option<i64>,
    pub name: String,
    pub description: String,
    pub created_at: String,
    pub settings: HashMap<String, String>, // Setting name -> value
}

/// Represents system hardware information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
    pub cpu_name: String,
    pub cpu_cores: u32,
    pub ram_gb: u64,
    pub gpu_name: String,
    pub gpu_vram_mb: u64,
}

impl Setting {
    /// Check if the provided value is valid for this setting
    pub fn is_valid_value(&self, value: &str) -> bool {
        match &self.value_type {
            SettingValueType::Boolean => {
                value == "0" || value == "1" || value == "True" || value == "False"
            }
            SettingValueType::Integer => value.parse::<i64>().is_ok(),
            SettingValueType::Float => value.parse::<f64>().is_ok(),
            SettingValueType::String => true,
            SettingValueType::Enum(options) => options.contains(&value.to_string()),
        }
    }
}