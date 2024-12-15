use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let swift_sources = ["swift/overlay.swift"];

    let status = Command::new("swiftc")
        .args(["-emit-library", "-static", "-module-name", "CartographerBridge", "-o"])
        .arg(out_dir.join("libcartographer_bridge.a"))
        .args(swift_sources)
        .status()
        .expect("swiftc not found. install xcode");

    assert!(status.success(), "swift compilation failed");

    println!("cargo:rustc-link-search=native={}", out_dir.display());
    println!("cargo:rustc-link-lib=static=cartographer_bridge");
    println!("cargo:rustc-link-lib=framework=AppKit");
    println!("cargo:rustc-link-lib=framework=Foundation");

    for src in &swift_sources {
        println!("cargo:rerun-if-changed={src}");
    }
}
