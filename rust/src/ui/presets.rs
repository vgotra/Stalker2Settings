// Presets module for Stalker2Settings
// Contains presets management functionality

use cursive::{
    Cursive,
    views::{Dialog, TextView, EditView, SelectView, LinearLayout, Button, DummyView},
    traits::{Nameable, Scrollable},
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
use super::AppData;

/// Show the list of presets
pub fn show_presets_list(app: &mut Cursive, db_conn: Rc<RefCell<Connection>>) {
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
pub fn show_preset_detail(app: &mut Cursive, preset: SettingsPreset, db_conn: Rc<RefCell<Connection>>) {
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
pub fn apply_preset(app: &mut Cursive, preset: SettingsPreset, _db_conn: Rc<RefCell<Connection>>) {
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
