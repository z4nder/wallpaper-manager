use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Serialize, Deserialize, Default)]
struct WallpaperState {
    applied: HashMap<String, String>,
    orientation: HashMap<String, Orientation>,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Copy, Debug)]
enum Orientation {
    Horizontal,
    Vertical,
}

fn get_state_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join("wallpaper-manager/state.json")
}

fn load_state() -> WallpaperState {
    let path = get_state_path();
    if let Ok(data) = fs::read_to_string(path) {
        serde_json::from_str(&data).unwrap_or_default()
    } else {
        WallpaperState::default()
    }
}

pub fn reapply_saved_wallpapers() {
    let state = load_state();

    for (monitor, image_path) in state.applied {
        if !Path::new(&image_path).exists() {
            eprintln!("⚠️ Image not found: {image_path}");
            continue;
        }

        let mut args = vec!["img", "--outputs", &monitor, &image_path];

        if let Some(Orientation::Vertical) = state.orientation.get(&monitor) {
            args.extend(["--resize", "fit"]);
        }

        match Command::new("swww").args(args).output() {
            Ok(output) if output.status.success() => {
                println!(
                    "✅ Applied wallpaper for {} ({:?})",
                    monitor,
                    state.orientation.get(&monitor)
                );
            }
            Ok(output) => {
                eprintln!(
                    "❌ Failed to apply wallpaper for {monitor}: {}",
                    String::from_utf8_lossy(&output.stderr)
                );
            }
            Err(err) => {
                eprintln!("❌ Error executing swww for {monitor}: {err}");
            }
        }
    }
}
