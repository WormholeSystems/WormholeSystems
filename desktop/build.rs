// The Windows exe carries the app icon as resource 1, which is where Explorer and GPUI's
// window class both look for it.
fn main() {
    println!("cargo:rerun-if-changed=assets/icon.ico");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("assets/icon.ico")
            .set("ProductName", "WormholeSystems")
            .set("FileDescription", "WormholeSystems");
        res.compile().expect("could not embed the icon");
    }
}
