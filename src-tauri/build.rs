fn main() {
    // Rebuild the asset version when any embedded Android engine asset changes.
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("android") {
        use std::hash::{Hash, Hasher};
        let root = std::path::PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
        let mut hash = std::collections::hash_map::DefaultHasher::new();
        for name in [
            "config.toml",
            "mix9svqfreestyle_bsmix.bin.lz4",
            "mix9svqstandard_bs15.bin.lz4",
            "mix9svqrenju_bs15_black.bin.lz4",
            "mix9svqrenju_bs15_white.bin.lz4",
            "model210901.bin",
        ] {
            let path = root.join("resources/engine").join(name);
            println!("cargo:rerun-if-changed={}", path.display());
            name.hash(&mut hash);
            std::fs::read(path).unwrap().hash(&mut hash);
        }
        println!("cargo:rustc-env=RAPFI_ASSET_VERSION={:016x}", hash.finish());
    }

    tauri_build::build();
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("android") {
        let root = std::path::PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
        let abi = match std::env::var("CARGO_CFG_TARGET_ARCH").unwrap().as_str() {
            "aarch64" => "arm64-v8a",
            "arm" => "armeabi-v7a",
            "x86" => "x86",
            "x86_64" => "x86_64",
            other => panic!("Unsupported Android architecture: {other}"),
        };
        println!(
            "cargo:rustc-link-search=native={}",
            root.join("gen/android/app/src/main/jniLibs")
                .join(abi)
                .display()
        );
        println!("cargo:rustc-link-lib=dylib=rapfi");
    }
}
