fn main() {
    // tauri build embeds common-controls on build but not test which errors the tests on windows
    // this fixes the windows test
    let is_msvc_windows = std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows")
        && std::env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok("msvc");
    if is_msvc_windows {
        let manifest = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("common-controls-v6.manifest");
        println!("cargo:rerun-if-changed=tests/common-controls-v6.manifest");
        println!("cargo:rustc-link-arg-tests=/MANIFEST:EMBED");
        println!(
            "cargo:rustc-link-arg-tests=/MANIFESTINPUT:{}",
            manifest.display()
        );
    }

    tauri_build::build()
}
