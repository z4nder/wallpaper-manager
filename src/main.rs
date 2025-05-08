// src/main.rs
use eframe::egui;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Default)]
struct WallpaperState {
    applied: HashMap<String, String>, // monitor -> image_path
}

fn get_state_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join("wallpaper-manager/state.json")
}

fn load_state() -> WallpaperState {
    let path = get_state_path();
    if let Ok(data) = std::fs::read_to_string(path) {
        serde_json::from_str(&data).unwrap_or_default()
    } else {
        WallpaperState::default()
    }
}

fn save_state(state: &WallpaperState) {
    let path = get_state_path();

    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    if let Ok(json) = serde_json::to_string_pretty(state) {
        let _ = std::fs::write(path, json);
    }
}

fn set_wallpaper(monitor: &str, image_path: &str, state: &mut WallpaperState) {
    let output = Command::new("swww")
        .args(["img", "--outputs", monitor, image_path])
        .output();

    match output {
        Ok(result) => {
            if result.status.success() {
                println!("✅ Wallpaper set on {monitor}: {image_path}");
                state
                    .applied
                    .insert(monitor.to_string(), image_path.to_string());
                save_state(state);
            } else {
                eprintln!(
                    "❌ swww failed: {}",
                    String::from_utf8_lossy(&result.stderr)
                );
            }
        }
        Err(err) => {
            eprintln!("❌ Failed to run swww: {err}");
        }
    }
}

fn list_monitors() -> Vec<String> {
    let output = Command::new("hyprctl")
        .arg("monitors")
        .output()
        .expect("failed to execute hyprctl");

    String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter_map(|line| {
            if line.contains("Monitor") {
                line.split_whitespace().nth(1).map(|s| s.to_string())
            } else {
                None
            }
        })
        .collect()
}

fn list_images(folder: &str) -> Vec<PathBuf> {
    match fs::read_dir(folder) {
        Ok(entries) => entries
            .filter_map(Result::ok)
            .map(|e| e.path())
            .filter(|p| {
                if let Some(ext) = p.extension() {
                    ["jpg", "jpeg", "png", "bmp", "webp"].contains(&ext.to_str().unwrap_or(""))
                } else {
                    false
                }
            })
            .collect(),
        Err(_) => vec![],
    }
}

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
    /// Launch the GUI
    Gui,
    /// Reapply wallpapers from saved state
    Apply,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Gui => {
            run_gui()?;
        }
        Commands::Apply => {
            reapply_saved_wallpapers();
        }
    }

    Ok(())
}
fn run_gui() -> Result<(), eframe::Error> {
    let mut state = load_state();

    let options = eframe::NativeOptions::default();
    eframe::run_simple_native("Wallpaper Manager", options, move |ctx, _| {
        egui::CentralPanel::default().show(ctx, |ui| {
            let monitors = list_monitors();
            let images_dir = dirs::config_dir()
                .unwrap_or_else(|| PathBuf::from("/tmp"))
                .join("backgrounds");

            let images = list_images(images_dir.to_str().unwrap_or("/tmp"));

            if images.is_empty() {
                ui.label("⚠️ Nenhuma imagem encontrada em ~/.config/backgrounds");
                return;
            }

            for image in &images {
                if let Some(path) = image.to_str() {
                    ui.horizontal(|ui| {
                        ui.label(path);

                        for monitor in &monitors {
                            let label = if state.applied.get(monitor) == Some(&path.to_string()) {
                                format!("✅ {}", monitor)
                            } else {
                                format!("Set on {}", monitor)
                            };

                            if ui.button(label).clicked() {
                                set_wallpaper(monitor, path, &mut state);
                            }
                        }
                    });
                }
            }
        });
    })
}

fn reapply_saved_wallpapers() {
    let state = load_state();

    for (monitor, image_path) in state.applied {
        if std::path::Path::new(&image_path).exists() {
            let _ = std::process::Command::new("swww")
                .args(["img", "--outputs", &monitor, &image_path])
                .output();
        }
    }
}
