use std::env;
use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    // out_dir = target/{profile}/build/<crate>/out
    // target_dir = target/{profile}
    let target_dir = out_dir
        .ancestors()
        .nth(3)
        .expect("Failed to find target directory");

    if env::var("CARGO_CFG_WINDOWS").is_ok() {
        // copy the DLLs if we're on Windows
        copy_dlls(&manifest_dir, target_dir);
    }

    // copy the resource folder
    copy_folder(&manifest_dir.join("res"), &target_dir.join("res"));

    // Tell the linker where to find the libraries
    println!("cargo:rustc-link-search=native=lib");
}

fn copy_dlls(src_dir: &Path, target_dir: &Path) {
    for entry in fs::read_dir(src_dir).expect("Failed to read project root") {
        let entry = entry.expect("Invalid directory entry");
        let path = entry.path();

        if path.extension().and_then(|e| e.to_str()) == Some("dll") {
            let filename = path.file_name().unwrap();
            let dest = target_dir.join(filename);

            fs::copy(&path, &dest)
                .unwrap_or_else(|_| panic!("Failed to copy {:?} to {:?}", path, dest));

            println!("cargo:rerun-if-changed={}", path.display());
        }
    }
}

fn copy_folder(src: &Path, dest: &Path) {
    if !src.exists() {
        return;
    }

    fs::create_dir_all(dest).expect("Failed to create directory");

    for entry in fs::read_dir(src).expect("Failed to read directory") {
        let entry = entry.expect("Invalid entry");
        let path = entry.path();
        let dest_path = dest.join(entry.file_name());

        if path.is_dir() {
            copy_folder(&path, &dest_path);
        } else {
            fs::copy(&path, &dest_path)
                .unwrap_or_else(|_| panic!("Failed to copy {:?} to {:?}", path, dest_path));

            println!("cargo:rerun-if-changed={}", path.display());
        }
    }
}
