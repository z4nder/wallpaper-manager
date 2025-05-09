use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

#[derive(Serialize, Deserialize, Default)]
pub struct WallpaperState {
    pub applied: HashMap<String, String>,
    pub orientation: HashMap<String, Orientation>,
    pub rotation: HashMap<String, Rotation>,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Copy, Debug)]
pub enum Orientation {
    Horizontal,
    Vertical,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Copy, Debug)]
pub enum Rotation {
    None,
    Deg180,
}

fn get_state_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join("wallpaper-manager/state.json")
}

pub fn load_state() -> WallpaperState {
    let path = get_state_path();
    if let Ok(data) = fs::read_to_string(&path) {
        serde_json::from_str(&data).unwrap_or_default()
    } else {
        WallpaperState::default()
    }
}

pub fn save_state(state: &WallpaperState) {
    let path = get_state_path();
    if let Some(p) = path.parent() {
        let _ = fs::create_dir_all(p);
    }
    if let Ok(json) = serde_json::to_string_pretty(state) {
        let _ = fs::write(path, json);
    }
}

pub fn set_wallpaper(monitor: &str, image_path: &str, state: &mut WallpaperState) {
    let mut deg = 0;
    if matches!(state.orientation.get(monitor), Some(Orientation::Vertical)) {
        deg += 90;
    }
    if matches!(state.rotation.get(monitor), Some(Rotation::Deg180)) {
        deg += 180;
    }

    let mut final_path = image_path.to_string();
    if deg % 360 != 0 {
        let tmp = std::env::temp_dir().join("wallpaper_rotated.jpg");
        let args = ["-rotate", &deg.to_string()];
        if let Ok(out) = Command::new("convert")
            .arg(image_path)
            .args(&args)
            .arg(tmp.to_str().unwrap())
            .output()
        {
            if out.status.success() {
                final_path = tmp.to_str().unwrap().to_string();
            }
        }
    }

    if let Ok(out) = Command::new("swww")
        .args(["img", "--outputs", monitor, &final_path])
        .output()
    {
        if out.status.success() {
            state
                .applied
                .insert(monitor.to_string(), image_path.to_string());
            save_state(state);
        } else {
            eprintln!(
                "❌ swww failed ({}): {}",
                monitor,
                String::from_utf8_lossy(&out.stderr)
            );
        }
    }
}
