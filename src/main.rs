// src/main.rs
mod apply;
mod gui;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "wallpaper-manager")]
#[command(about = "Manage wallpapers per monitor", version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Gui,
    Apply,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Gui => {
            gui::run_gui()?;
        }
        Commands::Apply => {
            apply::reapply_saved_wallpapers();
        }
    }

    Ok(())
}
