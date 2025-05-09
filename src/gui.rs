use eframe::egui;
use egui::{ColorImage, RichText, TextureHandle, TextureOptions};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
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
    let mut final_path = image_path.to_string();

    if let Some(Orientation::Vertical) = state.orientation.get(monitor) {
        let rotated_path = std::env::temp_dir().join("rotated_wallpaper.jpg");

        let convert_result = Command::new("convert")
            .args([image_path, "-rotate", "90", rotated_path.to_str().unwrap()])
            .output();

        if let Ok(output) = convert_result {
            if output.status.success() {
                final_path = rotated_path.to_str().unwrap().to_string();
            } else {
                eprintln!(
                    "❌ Failed to rotate image:\n{}",
                    String::from_utf8_lossy(&output.stderr)
                );
            }
        } else {
            eprintln!("❌ Failed to run `convert` to rotate image");
        }
    }

    let output = Command::new("swww")
        .args(["img", "--outputs", monitor, &final_path])
        .output();

    if let Ok(result) = output {
        if result.status.success() {
            println!("✅ Wallpaper set on {monitor}: {final_path}");
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
    } else {
        eprintln!("❌ Failed to run swww");
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
                    let ext = ext.to_string_lossy().to_lowercase();
                    ["jpg", "jpeg", "png", "bmp", "webp"].contains(&ext.as_str())
                } else {
                    false
                }
            })
            .collect(),
        Err(_) => vec![],
    }
}

pub fn run_gui() -> Result<(), eframe::Error> {
    let mut state = load_state();
    let mut rename_target: Option<PathBuf> = None;
    let mut delete_target: Option<PathBuf> = None;
    let mut current_orientation: Orientation = Orientation::Horizontal;
    let mut show_orientation_modal: Option<(String, String)> = None;
    let mut new_name = String::new();
    let mut textures: std::collections::HashMap<PathBuf, TextureHandle> = HashMap::new();

    let options = eframe::NativeOptions::default();
    eframe::run_simple_native("Wallpaper Manager", options, move |ctx, _| {
        egui::CentralPanel::default().show(ctx, |ui| {
            let monitors = list_monitors();
            let mut images_dir = dirs::config_dir()
                .unwrap_or_else(|| PathBuf::from("/tmp"))
                .join("backgrounds");

            if fs::symlink_metadata(&images_dir)
                .map(|m| m.file_type().is_symlink())
                .unwrap_or(false)
            {
                images_dir = PathBuf::from("/tmp/wallpapers");
                let _ = fs::create_dir_all(&images_dir);
            }

            let images = list_images(images_dir.to_str().unwrap_or("/tmp"));

            ui.heading("📂 Wallpaper Manager");

            if ui.button("📥 Import image").clicked() {
                if let Some(path) = rfd::FileDialog::new().pick_file() {
                    if let Some(file_name) = path.file_name() {
                        let target = images_dir.join(file_name);
                        match fs::copy(&path, &target) {
                            Ok(_) => println!("✅ Copied: {:?}", target),
                            Err(err) => eprintln!("❌ Failed to copy to {:?}: {err}", target),
                        }
                    } else {
                        eprintln!("❌ Invalid file name: {:?}", path);
                    }
                }
            }

            if images.is_empty() {
                ui.label("⚠️ No images found in ~/.config/backgrounds");
                return;
            }

            for image_path in &images {
                if !textures.contains_key(image_path) {
                    if let Ok(bytes) = fs::read(image_path) {
                        if let Ok(img) = image::load_from_memory(&bytes) {
                            let img = img.to_rgba8();
                            let size = [img.width() as usize, img.height() as usize];
                            let color_image = ColorImage::from_rgba_unmultiplied(size, &img);
                            let tex = ctx.load_texture(
                                image_path.to_string_lossy(),
                                color_image,
                                TextureOptions::default(),
                            );
                            textures.insert(image_path.clone(), tex);
                        }
                    }
                }
            }

            for image in &images {
                if let Some(path_str) = image.to_str() {
                    ui.horizontal(|ui| {
                        // 1) PREVIEW: thumbnail fixa de 80×80
                        if let Some(tex) = textures.get(image) {
                            ui.add(egui::Image::new(tex).max_width(200.0).rounding(10.0));
                        } else {
                            ui.add_sized([80.0, 80.0], egui::Label::new("🖼️"));
                        }

                        // 2) conteúdo à direita, em coluna
                        ui.vertical(|ui| {
                            // 2.1) nome do arquivo
                            ui.label(RichText::new(path_str).strong());

                            // 2.2) botões de setar em cada monitor, numa linha só
                            ui.horizontal_wrapped(|ui| {
                                for monitor in &monitors {
                                    let label = if state.applied.get(monitor)
                                        == Some(&path_str.to_string())
                                    {
                                        format!("✅ {}", monitor)
                                    } else {
                                        format!("Set {}", monitor)
                                    };
                                    if ui.small_button(&label).clicked() {
                                        show_orientation_modal =
                                            Some((monitor.clone(), path_str.to_string()));
                                        current_orientation = state
                                            .orientation
                                            .get(monitor)
                                            .copied()
                                            .unwrap_or(Orientation::Horizontal);
                                    }
                                }
                            });

                            // 2.3) ações de renomear/deletar
                            ui.horizontal(|ui| {
                                if ui.small_button("✏️ Rename").clicked() {
                                    rename_target = Some(image.clone());
                                    new_name = image
                                        .file_name()
                                        .unwrap_or_default()
                                        .to_string_lossy()
                                        .to_string();
                                }
                                if ui.small_button("🗑️ Delete").clicked() {
                                    delete_target = Some(image.clone());
                                }
                            });
                        });
                    });
                    ui.separator(); // separa visualmente cada linha
                }
            }

            if let Some((monitor, image_path)) = show_orientation_modal.clone() {
                egui::Window::new(format!("Orientation for {monitor}"))
                    .collapsible(false)
                    .show(ctx, |ui| {
                        ui.label("Choose orientation:");
                        ui.horizontal(|ui| {
                            ui.radio_value(
                                &mut current_orientation,
                                Orientation::Horizontal,
                                "Horizontal",
                            );
                            ui.radio_value(
                                &mut current_orientation,
                                Orientation::Vertical,
                                "Vertical",
                            );
                        });

                        if ui.button("Apply").clicked() {
                            state
                                .orientation
                                .insert(monitor.clone(), current_orientation);
                            set_wallpaper(&monitor, &image_path, &mut state);
                            show_orientation_modal = None;
                        }

                        if ui.button("Cancel").clicked() {
                            show_orientation_modal = None;
                        }
                    });
            }

            if let Some(target) = rename_target.clone() {
                egui::Window::new("Rename image")
                    .collapsible(false)
                    .show(ctx, |ui| {
                        ui.label("New file name:");
                        ui.text_edit_singleline(&mut new_name);

                        if ui.button("Save").clicked() && !new_name.trim().is_empty() {
                            let new_path = images_dir.join(&new_name);
                            if fs::rename(&target, new_path).is_ok() {
                                rename_target = None;
                            }
                        }

                        if ui.button("Cancel").clicked() {
                            rename_target = None;
                        }
                    });
            }

            if let Some(target) = delete_target.clone() {
                egui::Window::new("Confirm deletion")
                    .collapsible(false)
                    .show(ctx, |ui| {
                        ui.label(format!(
                            "Are you sure you want to delete {:?}?",
                            target.file_name()
                        ));
                        if ui.button("Yes").clicked() {
                            let _ = fs::remove_file(&target);
                            delete_target = None;
                        }
                        if ui.button("Cancel").clicked() {
                            delete_target = None;
                        }
                    });
            }
        });
    })
}
