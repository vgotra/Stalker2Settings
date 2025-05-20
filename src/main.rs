mod models;
mod db;
mod config;
mod ui;
mod system;

use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    // Initialize database
    let db_connection = db::initialize_db()?;

    // Start the UI application
    ui::run_app(db_connection)?;

    Ok(())
}
