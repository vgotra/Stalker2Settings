// UI module for Stalker2Settings
// Contains the main UI components and functionality

mod theme;
mod menu;
mod settings;
mod presets;

use cursive::{
    Cursive, CursiveExt,
    views::Dialog,
    views::TextView,
    event::Key,
};
use rusqlite::Connection;
use std::rc::Rc;
use std::cell::RefCell;
use std::error::Error;

use crate::models::{Setting, SettingsPreset, SystemInfo};
use crate::config;
use crate::system;

// Re-export main UI components
pub use self::menu::show_main_menu;
pub use self::settings::{show_settings_list, show_setting_detail};
pub use self::presets::{show_presets_list, show_preset_detail};
pub use self::theme::create_theme;

/// Application data stored in Cursive user_data
pub struct AppData {
    pub settings: Vec<Setting>,
    pub system_info: SystemInfo,
    pub current_preset: Option<SettingsPreset>,
}

/// Run the main application UI
pub fn run_app(db_connection: Connection) -> Result<(), Box<dyn Error>> {
    // Create a shared database connection
    let db_conn = Rc::new(RefCell::new(db_connection));

    // Create a cursive app
    let mut app = Cursive::default();

    // Set theme
    app.set_theme(theme::create_theme());

    // Add global callbacks
    app.add_global_callback(Key::Esc, |s| { s.pop_layer(); });
    app.add_global_callback('q', |s| { s.quit(); });

    // Load settings from database
    let settings = match config::load_settings_from_db(&db_conn.borrow()) {
        Ok(s) => s,
        Err(e) => {
            app.add_layer(
                Dialog::around(TextView::new(format!(
                    "Error loading settings from database: {}\n\nPlease make sure the database is properly initialized.",
                    e
                )))
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