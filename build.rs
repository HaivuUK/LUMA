fn main() {
    // Embed the icon + version info into the Windows .exe (Explorer, taskbar, Start menu).
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("icons/icon.ico");
        res.compile().expect("failed to embed Windows icon");
    }
    println!("cargo:rerun-if-changed=icons/icon.ico");
}
