use std::{env, fs, path::Path};

fn main() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR");
    let themes_dir = Path::new(&manifest_dir).join("assets/css/themes");
    println!("cargo:rerun-if-changed={}", themes_dir.display());
    if let Ok(entries) = fs::read_dir(&themes_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("css") {
                println!("cargo:rerun-if-changed={}", path.display());
            }
        }
    }
}
