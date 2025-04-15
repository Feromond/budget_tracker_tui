use std::{env, path::PathBuf};
use winresource::WindowsResource;

fn main() {
    // Only set the icon if building for Windows
    if env::var_os("CARGO_CFG_WINDOWS").is_some() {
        let icon_path =
            PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()).join("budget_tracker_icon.ico");
        if icon_path.exists() {
            WindowsResource::new()
                .set_icon(icon_path.to_str().unwrap())
                .compile()
                .expect("Failed to compile Windows resources");
            println!("Build: Windows icon set successfully from {:?}.", icon_path);
        } else {
            println!(
                "Build Warning: budget_tracker_icon.ico not found at {:?}, skipping icon setting.",
                icon_path
            );
        }
    }
}
