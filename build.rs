use std::env;
use std::fs;
use std::path::Path;

fn main() {
    if let Ok(target_env) = env::var("CARGO_CFG_TARGET_ENV") {
        if target_env == "msvc" {

            let ffmpeg_dir = env::var("FFMPEG_DIR").expect("ffmpeg_dir environment variable not set");

            let manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR environment variable not set");

            let profile = env::var("PROFILE").expect("PROFILE environment variable not set");
            let out_dir = if profile == "release" {
                Path::new(&manifest_dir).join("target").join("release")
            } else {
                Path::new(&manifest_dir).join("target").join("debug")
            };

            let ffmpeg_path = Path::new(&ffmpeg_dir).join("bin");
            let out_path = Path::new(&out_dir);

            for entry in fs::read_dir(ffmpeg_path).expect("Failed to read ffmpeg_dir") {
                let entry = entry.expect("Failed to get entry");
                let path = entry.path();

                if path.is_file() {
                    if let Some(extension) = path.extension() {
                        if extension == "dll" {
                            let dest_path = out_path.join(path.file_name().expect("Failed to get file name"));

                            fs::copy(&path, &dest_path).expect("Failed to copy file");
                        }
                    }
                }
            }
        } else {
        }
    } else {
    }
}
