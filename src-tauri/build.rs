use std::{env, path::PathBuf, process::Command};

fn main() {
    tauri_build::build();
    println!("cargo:rerun-if-changed=src/keychain.m");
    println!("cargo:rerun-if-env-changed=CARGO_FEATURE_E2E");
    if env::var_os("CARGO_FEATURE_E2E").is_some() {
        return;
    }
    if env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("macos") {
        let out = PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR"));
        let library = out.join("libgeld_keychain.a");
        let arch = env::var("CARGO_CFG_TARGET_ARCH").expect("CARGO_CFG_TARGET_ARCH");
        let deployment = if arch == "aarch64" { "11.0" } else { "10.15" };
        let arch = if arch == "aarch64" {
            "arm64"
        } else {
            arch.as_str()
        };
        let object = out.join("keychain.o");
        let status = Command::new("clang")
            .args(["-arch", arch, "-fobjc-arc", "-c", "src/keychain.m", "-o"])
            .arg(&object)
            .arg(format!("-mmacosx-version-min={deployment}"))
            .status()
            .expect("Clang is required to build macOS Keychain support");
        assert!(
            status.success(),
            "Objective-C Keychain bridge compilation failed"
        );
        let _ = std::fs::remove_file(&library);
        let status = Command::new("ar")
            .args(["crus"])
            .arg(&library)
            .arg(&object)
            .status()
            .expect("ar is required to archive macOS Keychain support");
        assert!(status.success(), "Objective-C Keychain archive failed");
        println!("cargo:rustc-link-search=native={}", out.display());
        println!("cargo:rustc-link-lib=static=geld_keychain");
        for framework in ["Foundation", "Security", "LocalAuthentication"] {
            println!("cargo:rustc-link-lib=framework={framework}");
        }
    }
}
