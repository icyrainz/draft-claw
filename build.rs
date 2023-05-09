use std::env;
use std::fs;
use std::path::Path;

fn main() {
    println!("cargo:rustc-link-lib=framework=ApplicationServices");

    // Get the target directory
    let target_dir = env::var("CARGO_TARGET_DIR").unwrap_or_else(|_| "target".to_string());
    let profile = env::var("PROFILE").unwrap_or_else(|_| "debug".to_string());

    let target_path = Path::new(&target_dir).join(&profile).join("resource");

    // Set the source and destination paths
    let src_dir = Path::new("resource");

    // Copy the files and directories recursively
    if let Err(e) = copy_dir(src_dir, &target_path) {
        eprintln!("Failed to copy files from {}: {}", src_dir.display(), e);
    }
}

fn copy_dir(src: &Path, dst: &Path) -> Result<(), Box<dyn std::error::Error>> {
    // Create the destination directory if it doesn't exist
    fs::create_dir_all(dst)?;

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if src_path.is_dir() {
            copy_dir(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
            println!("Copied {} to {}", src_path.display(), dst_path.display());
        }
    }

    Ok(())
}
