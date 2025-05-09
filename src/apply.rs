use crate::state::{load_state, Orientation};
use std::path::Path;
use std::process::Command;

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
