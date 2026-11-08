use std::env;
use std::fs;

/// The suite version in the repository root is the only source of truth.
/// `desktop/Cargo.toml` mirrors it because Cargo requires a literal version;
/// the build fails when the two drift apart.
fn workspace_version() -> String {
    let manifest =
        fs::read_to_string("../../Cargo.toml").expect("read the repository root Cargo.toml");
    let mut in_section = false;
    for line in manifest.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') {
            in_section = trimmed == "[workspace.metadata.ruxcmd]";
            continue;
        }
        if in_section && let Some(value) = trimmed.strip_prefix("version") {
            let value = value.trim_start();
            if let Some(value) = value.strip_prefix('=') {
                return value.trim().trim_matches('"').to_string();
            }
        }
    }
    panic!("[workspace.metadata.ruxcmd].version is missing from the root Cargo.toml");
}

fn main() {
    println!("cargo:rerun-if-changed=../../Cargo.toml");

    let version = workspace_version();
    let cargo_version = env::var("CARGO_PKG_VERSION").unwrap_or_default();
    if cargo_version != version {
        panic!(
            "desktop/src-tauri/Cargo.toml version ({cargo_version}) must match \
             workspace.metadata.ruxcmd.version ({version})"
        );
    }

    println!("cargo:rustc-env=RUXCMD_DESKTOP_VERSION={version}");
    println!(
        "cargo:rustc-env=RUXCMD_DESKTOP_TARGET={}",
        env::var("TARGET").unwrap_or_default()
    );
    let seconds = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|elapsed| elapsed.as_secs())
        .unwrap_or(0);
    println!("cargo:rustc-env=RUXCMD_DESKTOP_BUILD_UNIX={seconds}");

    // tauri-build reads this variable before writing the Windows version
    // resource, so the PE ProductVersion/FileVersion follow the suite version
    // instead of Cargo's package metadata.
    let merged = match env::var("TAURI_CONFIG") {
        Ok(existing) if !existing.trim().is_empty() => {
            let body = existing
                .trim()
                .trim_start_matches('{')
                .trim_end_matches('}');
            if body.trim().is_empty() {
                format!(r#"{{"version":"{version}"}}"#)
            } else {
                format!(r#"{{{body},"version":"{version}"}}"#)
            }
        }
        _ => format!(r#"{{"version":"{version}"}}"#),
    };
    // SAFETY: single-threaded build script, before tauri-build reads the value.
    unsafe { env::set_var("TAURI_CONFIG", merged) };

    tauri_build::build();
}
