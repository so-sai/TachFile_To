// libs/iron_python_bridge/build.rs
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    // Get the base directory of the project
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    
    // Attempt to find the top-level target directory
    // In complex workspaces, this can be tricky, so we use a robust heuristic
    let target_dir = out_dir
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap(); // This usually points to target/debug or target/release

    let src_python_dir = Path::new(&manifest_dir).join("python");
    let dest_python_dir = target_dir.join("python");

    println!("cargo:warning=Copying Python assets from {:?} to {:?}", src_python_dir, dest_python_dir);

    if src_python_dir.exists() {
        fs::create_dir_all(&dest_python_dir).expect("Failed to create python dir");

        // Copy extraction.py and other .py files
        for entry in fs::read_dir(src_python_dir).expect("Failed to read python dir") {
            let entry = entry.expect("Failed to read entry");
            let path = entry.path();
            if path.extension().map_or(false, |ext| ext == "py") {
                let dest = dest_python_dir.join(path.file_name().unwrap());
                fs::copy(&path, &dest).expect("Failed to copy .py file");
            }
        }
    }

    // Rerun if any python file changes
    println!("cargo:rerun-if-changed=python");
}
