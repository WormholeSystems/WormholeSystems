// The Windows exe carries the app icon as resource 1, which is where Explorer and GPUI's
// window class both look for it, and the dark one as 2 for the window to switch to.
fn main() {
    println!("cargo:rerun-if-changed=assets/icon.ico");
    println!("cargo:rerun-if-changed=assets/icon-dark.ico");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        let mut res = winresource::WindowsResource::new();
        res.set_icon_with_id("assets/icon.ico", "1")
            .set_icon_with_id("assets/icon-dark.ico", "2")
            .set("ProductName", "WormholeSystems")
            .set("FileDescription", &description());
        res.compile().expect("could not embed the icon");
    }
}

/// "WormholeSystems (alpha)" while the version is a pre-release, so Explorer and the task
/// manager say what the build is.
fn description() -> String {
    let version = std::env::var("CARGO_PKG_VERSION").unwrap_or_default();
    match version.split_once('-') {
        Some((_, pre)) => {
            let stage = pre.split('.').next().unwrap_or(pre);
            format!("WormholeSystems ({stage})")
        }
        None => "WormholeSystems".into(),
    }
}
