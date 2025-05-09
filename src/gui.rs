use eframe::{self, egui};
use egui::{Button, Color32, ColorImage, RichText, TextureHandle, TextureOptions, Visuals};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

use crate::state::{load_state, set_wallpaper, Orientation, Rotation, WallpaperState};

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
    let mut textures: HashMap<PathBuf, TextureHandle> = HashMap::new();
    let mut current_rotation = Rotation::None;

    let options = eframe::NativeOptions::default();
    eframe::run_simple_native("Wallpaper Manager", options, move |ctx, _| {
        ctx.set_visuals(Visuals::dark());
        let mut style = (*ctx.style()).clone();
        style.visuals.widgets.inactive.bg_fill = Color32::from_rgb(30, 30, 30);
        style.visuals.widgets.hovered.bg_fill = Color32::from_rgb(50, 50, 50);
        style.visuals.widgets.active.bg_fill = Color32::from_rgb(70, 70, 70);
        style.visuals.selection.bg_fill = Color32::from_rgb(0, 120, 200);
        ctx.set_style(style);

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

            ui.vertical_centered(|ui| {
                ui.heading(
                    RichText::new("📂 Wallpaper Manager")
                        .color(Color32::from_rgb(255, 255, 255))
                        .size(28.0),
                );
            });

            ui.add_space(10.0);

            if ui.button("📥 Import image").clicked() {
                if let Some(path) = rfd::FileDialog::new().pick_file() {
                    if let Some(file_name) = path.file_name() {
                        let target = images_dir.join(file_name);
                        match fs::copy(&path, &target) {
                            Ok(_) => println!("✅ Copied: {:?}", target),
                            Err(err) => eprintln!("❌ Failed to copy to {:?}: {err}", target),
                        }
                    }
                }
            }

            ui.separator();

            if images.is_empty() {
                ui.centered_and_justified(|ui| {
                    ui.label("⚠️ No images found in ~/.config/backgrounds");
                });
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
                        if let Some(tex) = textures.get(image) {
                            ui.add(egui::Image::new(tex).max_width(200.0).rounding(10.0));
                        } else {
                            ui.add_sized([80.0, 80.0], egui::Label::new("🖼️"));
                        }

                        ui.vertical(|ui| {
                            ui.label(RichText::new(path_str).color(Color32::LIGHT_GRAY));

                            ui.horizontal_wrapped(|ui| {
                                for monitor in &monitors {
                                    let is_applied =
                                        state.applied.get(monitor) == Some(&path_str.to_string());
                                    let label = if is_applied { "✅ Applied" } else { "Set" };
                                    let (bg, fg) = if is_applied {
                                        (Color32::from_rgb(20, 100, 20), Color32::WHITE)
                                    } else {
                                        (Color32::from_rgb(40, 40, 120), Color32::WHITE)
                                    };
                                    let btn = egui::Button::new(
                                        RichText::new(format!("{label} {monitor}")).color(fg),
                                    )
                                    .fill(bg)
                                    .rounding(5.0);
                                    if ui.add(btn).clicked() {
                                        show_orientation_modal =
                                            Some((monitor.clone(), path_str.to_string()));
                                        current_orientation = state
                                            .orientation
                                            .get(monitor)
                                            .copied()
                                            .unwrap_or(Orientation::Horizontal);
                                        current_rotation = state
                                            .rotation
                                            .get(monitor)
                                            .copied()
                                            .unwrap_or(Rotation::None);
                                    }
                                }
                            });

                            ui.horizontal(|ui| {
                                let btn_rename = Button::new(
                                    RichText::new("✏️ Rename")
                                        .color(Color32::from_rgb(255, 255, 255)),
                                )
                                .fill(Color32::from_rgb(200, 180, 50))
                                .rounding(5.0);
                                if ui.add(btn_rename).clicked() {
                                    rename_target = Some(image.clone());
                                    new_name = image
                                        .file_name()
                                        .unwrap_or_default()
                                        .to_string_lossy()
                                        .to_string();
                                }
                                let btn_delete = Button::new(
                                    RichText::new("🗑️ Delete")
                                        .color(Color32::from_rgb(255, 255, 255)),
                                )
                                .fill(Color32::from_rgb(120, 20, 20))
                                .rounding(5.0);
                                if ui.add(btn_delete).clicked() {
                                    delete_target = Some(image.clone());
                                }
                            });
                        });
                    });
                    ui.separator();
                }
            }

            if let Some((monitor, image_path)) = show_orientation_modal.clone() {
                egui::Window::new(format!("Configure for {monitor}"))
                    .collapsible(false)
                    .show(ctx, |ui| {
                        ui.label("Orientation:");
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

                        ui.label("Extra rotation:");
                        egui::ComboBox::from_id_source("rot_cb")
                            .selected_text(match current_rotation {
                                Rotation::None => "None",
                                Rotation::Deg180 => "180°",
                            })
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut current_rotation, Rotation::None, "None");
                                ui.selectable_value(
                                    &mut current_rotation,
                                    Rotation::Deg180,
                                    "180°",
                                );
                            });

                        ui.horizontal(|ui| {
                            if ui.button("Apply").clicked() {
                                state
                                    .orientation
                                    .insert(monitor.clone(), current_orientation);
                                state.rotation.insert(monitor.clone(), current_rotation);
                                set_wallpaper(&monitor, &image_path, &mut state);
                                show_orientation_modal = None;
                            }
                            if ui.button("Cancel").clicked() {
                                show_orientation_modal = None;
                            }
                        });
                    });
            }

            if let Some(target) = rename_target.clone() {
                egui::Window::new("Rename image")
                    .collapsible(false)
                    .show(ctx, |ui| {
                        ui.label("New file name:");
                        ui.text_edit_singleline(&mut new_name);

                        ui.horizontal(|ui| {
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
                        ui.horizontal(|ui| {
                            if ui.button("Yes").clicked() {
                                let _ = fs::remove_file(&target);
                                delete_target = None;
                            }
                            if ui.button("Cancel").clicked() {
                                delete_target = None;
                            }
                        });
                    });
            }
        });
    })
}
