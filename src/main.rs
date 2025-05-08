// src/main.rs
use eframe::egui;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn set_wallpaper(monitor: &str, image_path: &str) {
    let _ = Command::new("swww")
        .args(["img", image_path, "--output", monitor])
        .output();
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
    fs::read_dir(folder)
        .unwrap_or_default()
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| {
            if let Some(ext) = p.extension() {
                ["jpg", "jpeg", "png", "bmp", "webp"].contains(&ext.to_str().unwrap_or(""))
            } else {
                false
            }
        })
        .collect()
}

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions::default();
    eframe::run_simple_native("Wallpaper Manager", options, move |ctx, _| {
        egui::CentralPanel::default().show(ctx, |ui| {
            let monitors = list_monitors();
            let images = list_images("/home/gustavo/.config/backgrounds");

            for image in &images {
                if let Some(path) = image.to_str() {
                    ui.horizontal(|ui| {
                        ui.label(path);
                        for monitor in &monitors {
                            if ui.button(format!("Set on {}", monitor)).clicked() {
                                set_wallpaper(monitor, path);
                            }
                        }
                    });
                }
            }
        });
    })
}
