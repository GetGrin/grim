use std::process::Command;
use std::{env, fs};

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let tor_out_dir = format!("{}/tor", out_dir);
    let mut webtunnel_file = format!("{}/webtunnel", tor_out_dir);
    let exists = fs::exists(&webtunnel_file).unwrap();
    if !exists {
        // Create empty webtunnel file to allow build with include_bytes! macro.
        fs::create_dir(&tor_out_dir).unwrap_or_default();
        fs::File::create(&webtunnel_file).unwrap();
    }

    let target = env::var("TARGET").unwrap();
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap();

    let is_android = target_os == "android";
    if is_android {
        // Set a path to Android Webtunnel binary.
        let arch = if target.contains("aarch64") {
            "arm64-v8a"
        } else if target.contains("arm") {
            "armeabi-v7a"
        } else {
            "x86_64"
        };
        let root = env::var("CARGO_MANIFEST_DIR").unwrap();
        webtunnel_file = format!("{}/android/app/src/main/jniLibs/{}/libwebtunnel.so", root, arch);
    }

    // Build if Webtunnel binary is empty or not exists.
    let empty = match fs::File::open(&webtunnel_file) {
        Ok(file) => file.metadata().unwrap().len() == 0,
        Err(_) => true
    };
    let build = !exists || empty;
    if build {
        // Setup GOOS env variable.
        let go_os = if target_os == "macos" {
            "darwin"
        } else {
            target_os.as_str()
        };
        // Setup GOARCH env variable.
        let go_arch = if target.contains("aarch64") {
            "arm64"
        } else if target.contains("arm") {
            "arm"
        } else {
            "amd64"
        };
        // Run Webtunnel Go build.
        let output = if env::consts::OS == "windows" {
            Command::new("./scripts/webtunnel.bat")
                .arg(go_os)
                .arg(go_arch)
                .arg(webtunnel_file)
                .output()
        } else {
            Command::new("bash")
                .arg("./scripts/webtunnel.sh")
                .arg(go_os)
                .arg(go_arch)
                .arg(webtunnel_file)
                .output()
        };
        if let Ok(out) = output {
            if out.status.code().is_none() || out.status.code().unwrap() != 0 {
                panic!("webtunnel go build failed:\n{:?}", out);
            }
        }
    }
}