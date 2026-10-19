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
            .set("FileDescription", "WormholeSystems");
        res.compile().expect("could not embed the icon");
    }
}
