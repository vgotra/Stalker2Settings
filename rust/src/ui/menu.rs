// Menu module for Stalker2Settings
// Contains main menu functionality

use cursive::{
    Cursive,
    views::{Dialog, TextView, LinearLayout, Button, DummyView},
    align::HAlign,
    view::Resizable,
};
use rusqlite::Connection;
use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;

use crate::models::SettingsPreset;
use crate::db;
use crate::config;
use crate::system;
use super::AppData;
use super::presets::apply_preset;

/// Show the main menu
pub fn show_main_menu(app: &mut Cursive, db_conn: Rc<RefCell<Connection>>) {
    let system_info = app.user_data::<AppData>().unwrap().system_info.clone();
    let performance_tier = system::get_performance_tier(&system_info);

    let db_conn1 = Rc::clone(&db_conn);
    let db_conn2 = Rc::clone(&db_conn);
    let db_conn3 = Rc::clone(&db_conn);
    let db_conn4 = Rc::clone(&db_conn);

    app.add_layer(
        Dialog::around(
            LinearLayout::vertical()
                .child(TextView::new("STALKER 2 Settings Manager")
                    .h_align(HAlign::Center))
                .child(DummyView.fixed_height(1))
                .child(TextView::new(format!(
                    "System: {} with {} cores, {}GB RAM, {}",
                    system_info.cpu_name, system_info.cpu_cores, 
                    system_info.ram_gb, system_info.gpu_name
                )))
                .child(TextView::new(format!(
                    "Performance tier: {} (estimated VRAM: {}MB)",
                    performance_tier, system_info.gpu_vram_mb
                )))
                .child(DummyView.fixed_height(1))
                .child(Button::new("Edit Settings", move |s| {
                    super::settings::show_settings_list(s, Rc::clone(&db_conn1));
                }))
                .child(Button::new("Manage Presets", move |s| {
                    super::presets::show_presets_list(s, Rc::clone(&db_conn2));
                }))
                .child(Button::new("Generate Recommended Settings", move |s| {
                    generate_recommended_settings(s, Rc::clone(&db_conn3));
                }))
                .child(Button::new("Save Current Settings", move |s| {
                    save_current_settings(s, Rc::clone(&db_conn4));
                }))
                .child(Button::new("Quit", |s| { s.quit(); }))
        )
        .title("Main Menu")
        .min_width(60)
    );
}

/// Generate recommended settings based on system info
fn generate_recommended_settings(app: &mut Cursive, db_conn: Rc<RefCell<Connection>>) {
    let system_info = {
        let app_data = app.user_data::<AppData>().unwrap();
        app_data.system_info.clone()
    };

    // Generate recommended settings using the database connection
    let preset = config::generate_recommended_settings(&system_info, Some(&db_conn.borrow()));
    let preset_clone = preset.clone();
    let db_conn_clone = Rc::clone(&db_conn);

    // Show confirmation dialog
    app.add_layer(
        Dialog::around(
            LinearLayout::vertical()
                .child(TextView::new(format!(
                    "Generated recommended settings for your system:\n{} with {} cores, {}GB RAM",
                    system_info.cpu_name, system_info.cpu_cores, system_info.ram_gb
                )))
                .child(TextView::new(format!(
                    "Performance tier: {}",
                    system::get_performance_tier(&system_info)
                )))
                .child(DummyView.fixed_height(1))
                .child(TextView::new("Do you want to apply these settings?"))
        )
        .title("Recommended Settings")
        .button("Apply", move |s| {
            // Fix: Use a modified version of apply_preset that doesn't pop too many layers
            apply_recommended_preset(s, preset_clone.clone(), Rc::clone(&db_conn_clone));
        })
        .button("Cancel", |s| { s.pop_layer(); })
        .min_width(60)
    );
}

/// Apply a recommended preset without popping too many layers
fn apply_recommended_preset(app: &mut Cursive, preset: SettingsPreset, _db_conn: Rc<RefCell<Connection>>) {
    let app_data = app.user_data::<AppData>().unwrap();

    // Update settings with values from the preset
    for setting in &mut app_data.settings {
        let key = format!("{}.{}", setting.section, setting.name);
        if let Some(value) = preset.settings.get(&key) {
            setting.current_value = value.clone();
        }
    }

    // Set as current preset
    app_data.current_preset = Some(preset.clone());

    // Generate Engine.ini file
    if let Err(e) = config::generate_engine_ini_from_preset(&preset) {
        app.add_layer(
            Dialog::around(TextView::new(format!("Error generating Engine.ini: {}", e)))
                .title("Error")
                .button("OK", |s| { s.pop_layer(); })
        );
        return;
    }

    app.add_layer(
        Dialog::around(TextView::new(format!(
            "Applied preset '{}' and generated Engine.ini", preset.name
        )))
        .title("Success")
        .button("OK", move |s| { 
            s.pop_layer(); // Close this dialog
            s.pop_layer(); // Close confirmation dialog
            // Don't pop the main menu
        })
    );
}

/// Save current settings to Engine.ini
fn save_current_settings(app: &mut Cursive, _db_conn: Rc<RefCell<Connection>>) {
    let app_data = app.user_data::<AppData>().unwrap();

    // Create a preset from current settings
    let mut settings = HashMap::new();

    for setting in &app_data.settings {
        let key = format!("{}.{}", setting.section, setting.name);
        settings.insert(key, setting.current_value.clone());
    }

    let preset = SettingsPreset {
        id: None,
        name: String::from("Current"),
        description: String::from("Current settings"),
        created_at: chrono::Local::now().to_rfc3339(),
        settings,
    };

    // Generate Engine.ini file
    if let Err(e) = config::generate_engine_ini_from_preset(&preset) {
        app.add_layer(
            Dialog::around(TextView::new(format!("Error generating Engine.ini: {}", e)))
                .title("Error")
                .button("OK", |s| { s.pop_layer(); })
        );
        return;
    }

    app.add_layer(
        Dialog::around(TextView::new("Settings saved to Engine.ini"))
            .title("Success")
            .button("OK", |s| { s.pop_layer(); })
    );
}