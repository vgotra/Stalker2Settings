mod models;
mod db;
mod config;
mod ui;
mod system;
mod migrations;

use std::error::Error;
use clap::Parser;

/// Stalker 2 Settings Manager
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Recreate the database from scratch
    #[arg(short, long)]
    recreatedb: bool,
}

fn main() -> Result<(), Box<dyn Error>> {
    // Parse command line arguments
    let args = Args::parse();

    // Initialize database
    let db_connection = db::initialize_db(args.recreatedb)?;

    // Start the UI application
    ui::run_app(db_connection)?;

    Ok(())
}
