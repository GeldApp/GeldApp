use std::{env, path::PathBuf, process::Command};

fn main() {
    tauri_build::build();
    println!("cargo:rerun-if-changed=src/keychain.swift");
    if env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("macos") {
        let out = PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR"));
        let library = out.join("libgeld_keychain.a");
        let status = Command::new("swiftc")
            .args(["-emit-library", "-static", "-module-cache-path"])
            .arg(out.join("swift-module-cache"))
            .arg("src/keychain.swift")
            .arg("-o")
            .arg(&library)
            .status()
            .expect("Swift compiler is required to build macOS Keychain support");
        assert!(status.success(), "Swift Keychain bridge compilation failed");
        println!("cargo:rustc-link-search=native={}", out.display());
        println!("cargo:rustc-link-lib=static=geld_keychain");
        for framework in ["Foundation", "Security", "LocalAuthentication"] {
            println!("cargo:rustc-link-lib=framework={framework}");
        }
    }
}
