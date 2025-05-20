use cursive::{
    Cursive, CursiveExt,
    views::{Dialog, TextView, EditView, SelectView, LinearLayout, ScrollView, Button, DummyView},
    traits::{Nameable, Scrollable},
    align::HAlign,
    event::Key,
    theme::{BorderStyle, Theme, Color, PaletteColor},
    view::Resizable,
};
use rusqlite::Connection;
use std::rc::Rc;
use std::cell::RefCell;
use std::error::Error;
use std::collections::HashMap;

use crate::models::{Setting, SettingsPreset, SettingValueType, SystemInfo};
use crate::db;
use crate::config;
use crate::system;

/// Run the main application UI
pub fn run_app(db_connection: Connection) -> Result<(), Box<dyn Error>> {
    // Create a shared database connection
    let db_conn = Rc::new(RefCell::new(db_connection));

    // Create a cursive app
    let mut app = Cursive::default();

    // Set theme
    app.set_theme(create_theme());

    // Add global callbacks
    app.add_global_callback(Key::Esc, |s| { s.pop_layer(); });
    app.add_global_callback('q', |s| { s.quit(); });

    // Load settings from Engine.ini
    let settings = match config::load_engine_settings() {
        Ok(s) => s,
        Err(e) => {
            app.add_layer(
                Dialog::around(TextView::new(format!("Error loading settings: {}", e)))
                    .title("Error")
                    .button("Quit", |s| { s.quit(); })
            );
            app.run();
            return Err(Box::new(e));
        }
    };

    // Get system info
    let system_info = system::get_system_info();

    // Store settings and system info in user data
    app.set_user_data(AppData {
        settings,
        system_info,
        current_preset: None,
    });

    // Show main menu
    show_main_menu(&mut app, db_conn);

    // Run the application
    app.run();

    Ok(())
}

/// Application data stored in Cursive user_data
struct AppData {
    settings: Vec<Setting>,
    system_info: SystemInfo,
    current_preset: Option<SettingsPreset>,
}

/// Create a custom theme for the application
fn create_theme() -> Theme {
    let mut theme = Theme::default();
    theme.shadow = false;
    theme.borders = BorderStyle::Simple;
    theme.palette[PaletteColor::Background] = Color::TerminalDefault;
    theme.palette[PaletteColor::View] = Color::TerminalDefault;
    theme.palette[PaletteColor::Primary] = Color::Dark(cursive::theme::BaseColor::Blue);
    theme.palette[PaletteColor::Secondary] = Color::Light(cursive::theme::BaseColor::Blue);
    theme.palette[PaletteColor::TitlePrimary] = Color::Light(cursive::theme::BaseColor::Blue);
    theme.palette[PaletteColor::Highlight] = Color::Dark(cursive::theme::BaseColor::Blue);
    theme
}

/// Show the main menu
fn show_main_menu(app: &mut Cursive, db_conn: Rc<RefCell<Connection>>) {
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
                    show_settings_list(s, Rc::clone(&db_conn1));
                }))
                .child(Button::new("Manage Presets", move |s| {
                    show_presets_list(s, Rc::clone(&db_conn2));
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

/// Show the list of settings
fn show_settings_list(app: &mut Cursive, db_conn: Rc<RefCell<Connection>>) {
    let settings = app.user_data::<AppData>().unwrap().settings.clone();

    // Group settings by section
    let mut sections: Vec<String> = settings.iter()
        .map(|s| s.section.clone())
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();
    sections.sort();

    let mut select = SelectView::new()
        .h_align(HAlign::Left);

    for setting in &settings {
        let label = format!("[{}] {} = {}", setting.section, setting.name, setting.current_value);
        select.add_item(label, setting.clone());
    }

    let db_conn_clone = Rc::clone(&db_conn);
    select.set_on_submit(move |s, setting: &Setting| {
        show_setting_detail(s, setting.clone(), Rc::clone(&db_conn_clone));
    });

    app.add_layer(
        Dialog::around(
            select.scrollable()
        )
        .title("Settings")
        .button("Back", |s| { s.pop_layer(); })
        .min_width(70)
        .min_height(20)
    );
}

/// Show details for a specific setting
fn show_setting_detail(app: &mut Cursive, setting: Setting, db_conn: Rc<RefCell<Connection>>) {
    let mut content = LinearLayout::vertical()
        .child(TextView::new(format!("Setting: {}", setting.name)))
        .child(TextView::new(format!("Section: {}", setting.section)))
        .child(TextView::new(format!("Description: {}", setting.description)))
        .child(TextView::new(format!("Impact: {}", setting.impact)))
        .child(TextView::new(format!("Current Value: {}", setting.current_value)))
        .child(DummyView.fixed_height(1));

    // Add appropriate input based on value type
    let id = "value_input";
    match &setting.value_type {
        SettingValueType::Boolean => {
            let mut select = SelectView::new();
            select.add_item("0 (Disabled)", "0".to_string());
            select.add_item("1 (Enabled)", "1".to_string());

            // Set current value
            if setting.current_value == "1" || setting.current_value == "True" {
                select.set_selection(1);
            } else {
                select.set_selection(0);
            }

            content.add_child(TextView::new("New Value:"));
            content.add_child(select.with_name(id));
        },
        SettingValueType::Enum(options) => {
            let mut select = SelectView::new();
            for (i, option) in options.iter().enumerate() {
                select.add_item(option.clone(), option.clone());
                if option == &setting.current_value {
                    select.set_selection(i);
                }
            }

            content.add_child(TextView::new("New Value:"));
            content.add_child(select.with_name(id));
        },
        _ => {
            content.add_child(TextView::new("New Value:"));
            content.add_child(
                EditView::new()
                    .content(setting.current_value.clone())
                    .with_name(id)
            );
        }
    }

    // Add min/max value info if available
    if let Some(min) = &setting.min_value {
        content.add_child(TextView::new(format!("Minimum Value: {}", min)));
    }
    if let Some(max) = &setting.max_value {
        content.add_child(TextView::new(format!("Maximum Value: {}", max)));
    }

    let setting_clone = setting.clone();
    let db_conn_clone = Rc::clone(&db_conn);

    app.add_layer(
        Dialog::around(content.scrollable())
            .title(format!("Edit Setting: {}", setting.name))
            .button("Save", move |s| {
                save_setting_value(s, setting_clone.clone(), Rc::clone(&db_conn_clone));
            })
            .button("Cancel", |s| { s.pop_layer(); })
            .min_width(60)
    );
}

/// Save a setting value
fn save_setting_value(app: &mut Cursive, mut setting: Setting, db_conn: Rc<RefCell<Connection>>) {
    let id = "value_input";
    let new_value = match setting.value_type {
        SettingValueType::Boolean | SettingValueType::Enum(_) => {
            app.call_on_name(id, |view: &mut SelectView<String>| {
                view.selection()
                    .map(|value| (*value).clone())
                    .unwrap_or_default()
            }).unwrap_or_default()
        },
        _ => {
            app.call_on_name(id, |view: &mut EditView| {
                view.get_content().to_string()
            }).unwrap_or_default()
        }
    };

    // Validate the value
    if !setting.is_valid_value(&new_value) {
        app.add_layer(
            Dialog::around(TextView::new("Invalid value for this setting type."))
                .title("Error")
                .button("OK", |s| { s.pop_layer(); })
        );
        return;
    }

    // Update the setting in the app data
    setting.current_value = new_value;

    let mut app_data = app.user_data::<AppData>().unwrap();
    for s in &mut app_data.settings {
        if s.section == setting.section && s.name == setting.name {
            s.current_value = setting.current_value.clone();
            break;
        }
    }

    // Close the dialog
    app.pop_layer();

    // Refresh the settings list
    app.pop_layer();
    show_settings_list(app, db_conn);
}

/// Show the list of presets
fn show_presets_list(app: &mut Cursive, db_conn: Rc<RefCell<Connection>>) {
    // Load presets from database
    let presets = match db::get_all_presets(&db_conn.borrow()) {
        Ok(p) => p,
        Err(e) => {
            app.add_layer(
                Dialog::around(TextView::new(format!("Error loading presets: {}", e)))
                    .title("Error")
                    .button("OK", |s| { s.pop_layer(); })
            );
            return;
        }
    };

    let mut select = SelectView::new()
        .h_align(HAlign::Left);

    for preset in presets {
        let label = format!("{} - {}", preset.name, preset.description);
        select.add_item(label, preset);
    }

    let db_conn_clone = Rc::clone(&db_conn);
    select.set_on_submit(move |s, preset: &SettingsPreset| {
        show_preset_detail(s, preset.clone(), Rc::clone(&db_conn_clone));
    });

    let db_conn_clone = Rc::clone(&db_conn);

    app.add_layer(
        Dialog::around(
            LinearLayout::vertical()
                .child(select.scrollable())
                .child(DummyView.fixed_height(1))
                .child(
                    LinearLayout::horizontal()
                        .child(Button::new("New Preset", move |s| {
                            create_new_preset(s, Rc::clone(&db_conn_clone));
                        }))
                        .child(DummyView.fixed_width(1))
                        .child(Button::new("Back", |s| { s.pop_layer(); }))
                )
        )
        .title("Presets")
        .min_width(60)
    );
}

/// Show details for a specific preset
fn show_preset_detail(app: &mut Cursive, preset: SettingsPreset, db_conn: Rc<RefCell<Connection>>) {
    let preset_name = preset.name.clone();
    let preset_clone = preset.clone();
    let db_conn_clone1 = Rc::clone(&db_conn);
    let db_conn_clone2 = Rc::clone(&db_conn);

    app.add_layer(
        Dialog::around(
            LinearLayout::vertical()
                .child(TextView::new(format!("Name: {}", preset.name)))
                .child(TextView::new(format!("Description: {}", preset.description)))
                .child(TextView::new(format!("Created: {}", preset.created_at)))
                .child(DummyView.fixed_height(1))
                .child(TextView::new(format!("Settings: {} items", preset.settings.len())))
                .child(DummyView.fixed_height(1))
                .child(
                    LinearLayout::horizontal()
                        .child(Button::new("Apply", move |s| {
                            apply_preset(s, preset_clone.clone(), Rc::clone(&db_conn_clone1));
                        }))
                        .child(DummyView.fixed_width(1))
                        .child(Button::new("Delete", move |s| {
                            delete_preset(s, preset.clone(), Rc::clone(&db_conn_clone2));
                        }))
                        .child(DummyView.fixed_width(1))
                        .child(Button::new("Back", |s| { s.pop_layer(); }))
                )
        )
        .title(format!("Preset: {}", preset_name))
        .min_width(60)
    );
}

/// Apply a preset to the current settings
fn apply_preset(app: &mut Cursive, preset: SettingsPreset, db_conn: Rc<RefCell<Connection>>) {
    let mut app_data = app.user_data::<AppData>().unwrap();

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
            s.pop_layer(); // Close preset detail
            s.pop_layer(); // Close presets list
        })
    );
}

/// Delete a preset
fn delete_preset(app: &mut Cursive, preset: SettingsPreset, db_conn: Rc<RefCell<Connection>>) {
    if let Some(id) = preset.id {
        match db::delete_preset(&mut db_conn.borrow_mut(), id) {
            Ok(_) => {
                let db_conn_clone = Rc::clone(&db_conn);
                app.add_layer(
                    Dialog::around(TextView::new(format!(
                        "Deleted preset '{}'", preset.name
                    )))
                    .title("Success")
                    .button("OK", move |s| { 
                        s.pop_layer(); // Close this dialog
                        s.pop_layer(); // Close preset detail

                        // Refresh presets list
                        show_presets_list(s, Rc::clone(&db_conn_clone));
                    })
                );
            },
            Err(e) => {
                app.add_layer(
                    Dialog::around(TextView::new(format!("Error deleting preset: {}", e)))
                        .title("Error")
                        .button("OK", |s| { s.pop_layer(); })
                );
            }
        }
    }
}

/// Create a new preset
fn create_new_preset(app: &mut Cursive, db_conn: Rc<RefCell<Connection>>) {
    app.add_layer(
        Dialog::around(
            LinearLayout::vertical()
                .child(TextView::new("Preset Name:"))
                .child(EditView::new().with_name("preset_name"))
                .child(DummyView.fixed_height(1))
                .child(TextView::new("Description:"))
                .child(EditView::new().with_name("preset_description"))
        )
        .title("Create New Preset")
        .button("Save", {
            let db_conn_clone = Rc::clone(&db_conn);
            move |s| {
                let name = s.call_on_name("preset_name", |view: &mut EditView| {
                    view.get_content().to_string()
                }).unwrap_or_default();

                let description = s.call_on_name("preset_description", |view: &mut EditView| {
                    view.get_content().to_string()
                }).unwrap_or_default();

                if name.is_empty() {
                    s.add_layer(
                        Dialog::around(TextView::new("Preset name cannot be empty"))
                            .title("Error")
                            .button("OK", |s| { s.pop_layer(); })
                    );
                    return;
                }

                // Create preset from current settings
                let app_data = s.user_data::<AppData>().unwrap();
                let mut settings = HashMap::new();

                for setting in &app_data.settings {
                    let key = format!("{}.{}", setting.section, setting.name);
                    settings.insert(key, setting.current_value.clone());
                }

                let preset = SettingsPreset {
                    id: None,
                    name,
                    description,
                    created_at: chrono::Local::now().to_rfc3339(),
                    settings,
                };

                // Save to database
                let db_conn_clone2 = Rc::clone(&db_conn_clone);
                match db::save_preset(&mut db_conn_clone.borrow_mut(), &preset) {
                    Ok(_) => {
                        s.add_layer(
                            Dialog::around(TextView::new("Preset saved successfully"))
                                .title("Success")
                                .button("OK", move |s| { 
                                    s.pop_layer(); // Close this dialog
                                    s.pop_layer(); // Close new preset dialog

                                    // Refresh presets list
                                    show_presets_list(s, Rc::clone(&db_conn_clone2));
                                })
                        );
                    },
                    Err(e) => {
                        s.add_layer(
                            Dialog::around(TextView::new(format!("Error saving preset: {}", e)))
                                .title("Error")
                                .button("OK", |s| { s.pop_layer(); })
                        );
                    }
                }
            }
        })
        .button("Cancel", |s| { s.pop_layer(); })
        .min_width(50)
    );
}

/// Generate recommended settings based on system info
fn generate_recommended_settings(app: &mut Cursive, db_conn: Rc<RefCell<Connection>>) {
    let system_info = {
        let app_data = app.user_data::<AppData>().unwrap();
        app_data.system_info.clone()
    };

    // Generate recommended settings
    let preset = config::generate_recommended_settings(&system_info);
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
            apply_preset(s, preset_clone.clone(), Rc::clone(&db_conn_clone));
        })
        .button("Cancel", |s| { s.pop_layer(); })
        .min_width(60)
    );
}

/// Save current settings to Engine.ini
fn save_current_settings(app: &mut Cursive, db_conn: Rc<RefCell<Connection>>) {
    let app_data = app.user_data::<AppData>().unwrap();

    // Create a preset from current settings
    let mut settings = HashMap::new();

    for setting in &app_data.settings {
        let key = format!("{}.{}", setting.section, setting.name);
        settings.insert(key, setting.current_value.clone());
    }

    let preset = SettingsPreset {
        id: None,
        name: "Current".to_string(),
        description: "Current settings".to_string(),
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
