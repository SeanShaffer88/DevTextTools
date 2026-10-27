// Embeds the app icon into the Windows .exe (Explorer/taskbar file icon).
// No-op on non-Windows targets, and quietly skips if windres is missing.
fn main() {
    println!("cargo:rerun-if-changed=assets/icon.ico");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }
    let manifest = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_default();
    let out = std::env::var("OUT_DIR").unwrap_or_default();
    if manifest.is_empty() || out.is_empty() {
        return;
    }
    let windres = if std::env::var("CARGO_CFG_TARGET_ARCH").as_deref() == Ok("x86_64") {
        "x86_64-w64-mingw32-windres"
    } else {
        "windres"
    };
    let rc = format!("{out}/icon.rc");
    if std::fs::write(&rc, format!("1 ICON \"{manifest}/assets/icon.ico\"\n")).is_err() {
        return;
    }
    let obj = format!("{out}/icon.o");
    match std::process::Command::new(windres)
        .args([&rc, "-O", "coff", "-o", &obj])
        .status()
    {
        Ok(s) if s.success() => println!("cargo:rustc-link-arg={obj}"),
        _ => println!("cargo:warning={windres} not available; exe file icon skipped"),
    }
}
