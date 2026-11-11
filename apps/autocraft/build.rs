//! Windows only: embed the app icon and version info into `autocraft.exe`. Elsewhere a no-op.

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=../../assets/app-icon/autocraft.ico");
    println!("cargo:rerun-if-env-changed=AUTOCRAFT_REQUIRE_WINRES");
    println!("cargo:rerun-if-env-changed=AUTOCRAFT_BUILD_SHA");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }
    let mut res = winresource::WindowsResource::new();
    if std::path::Path::new("../../assets/app-icon/autocraft.ico").exists() {
        res.set_icon("../../assets/app-icon/autocraft.ico");
    }
    res.set("ProductName", "AutoCraft")
        .set("FileDescription", "AutoCraft computer-aided design")
        .set("CompanyName", "AutoCraft")
        .set("LegalCopyright", "Copyright (c) 2026 ArtCraft Team and the AutoCraft contributors. MIT OR Apache-2.0.")
        .set("OriginalFilename", "autocraft.exe")
        .set("InternalName", "autocraft");
    if let Err(e) = res.compile() {
        if std::env::var_os("AUTOCRAFT_REQUIRE_WINRES").is_some() {
            eprintln!("embedding Windows resources failed: {e}");
            std::process::exit(1);
        }
        println!("cargo:warning=autocraft.exe built without icon/version resources: {e}");
    }
}
