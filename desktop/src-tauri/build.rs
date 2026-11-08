fn main() {
    println!(
        "cargo:rustc-env=RUXCMD_DESKTOP_VERSION={}",
        std::env::var("CARGO_PKG_VERSION").unwrap_or_else(|_| "0.0.0".to_string())
    );
    println!(
        "cargo:rustc-env=RUXCMD_DESKTOP_TARGET={}",
        std::env::var("TARGET").unwrap_or_default()
    );
    let seconds = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|elapsed| elapsed.as_secs())
        .unwrap_or(0);
    println!("cargo:rustc-env=RUXCMD_DESKTOP_BUILD_UNIX={seconds}");

    tauri_build::build();
}
