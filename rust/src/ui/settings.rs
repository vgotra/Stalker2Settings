// Settings module for Stalker2Settings
// Contains settings management functionality

use cursive::{
    Cursive,
    views::{Dialog, TextView, EditView, SelectView, LinearLayout, DummyView},
    traits::{Nameable, Scrollable},
    align::HAlign,
    view::Resizable,
};
use rusqlite::Connection;
use std::rc::Rc;
use std::cell::RefCell;

use crate::models::{Setting, SettingValueType};
use crate::config;
use super::AppData;

/// Show the list of settings
pub fn show_settings_list(app: &mut Cursive, db_conn: Rc<RefCell<Connection>>) {
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
pub fn show_setting_detail(app: &mut Cursive, setting: Setting, db_conn: Rc<RefCell<Connection>>) {
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
            select.add_item("0 (Disabled)", String::from("0"));
            select.add_item("1 (Enabled)", String::from("1"));

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

    if !setting.is_valid_value(&new_value) {
        app.add_layer(
            Dialog::around(TextView::new("Invalid value for this setting type."))
                .title("Error")
                .button("OK", |s| { s.pop_layer(); })
        );
        return;
    }

    setting.current_value = new_value;

    // Save to database
    if let Err(e) = config::save_setting_to_db(&mut db_conn.borrow_mut(), &setting) {
        app.add_layer(
            Dialog::around(TextView::new(format!("Error saving setting: {}", e)))
                .title("Error")
                .button("OK", |s| { s.pop_layer(); })
        );
        return;
    }

    let app_data = app.user_data::<AppData>().unwrap();
    for s in &mut app_data.settings {
        if s.section == setting.section && s.name == setting.name {
            s.current_value = setting.current_value.clone();
            break;
        }
    }

    app.pop_layer();
    app.pop_layer();
    show_settings_list(app, db_conn);
}
