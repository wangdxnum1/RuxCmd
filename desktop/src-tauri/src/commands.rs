use std::sync::{Arc, Mutex};

use tauri::State;
use tauri_plugin_clipboard_manager::ClipboardExt;

use crate::detect;
use crate::integrity;
use crate::model::{
    InstallInfo, ManagerInfo, UpdateCheck, UpdateCheckError, VerificationReport,
};
use crate::release::{self, ReleaseCache};
use crate::util;

#[derive(Default)]
pub struct AppState {
    pub cache: Mutex<ReleaseCache>,
}

/// Only allow the domains the design lists for the desktop client.
const ALLOWED_EXTERNAL_HOSTS: [&str; 3] = ["api.github.com", "github.com", "objects.githubusercontent.com"];

#[tauri::command]
pub fn get_install_info() -> InstallInfo {
    detect::detect_installation()
}

#[tauri::command]
pub fn get_manager_info() -> ManagerInfo {
    ManagerInfo {
        version: env!("RUXCMD_DESKTOP_VERSION").to_string(),
        target: env!("RUXCMD_DESKTOP_TARGET").to_string(),
        build_time: util::unix_to_iso(env!("RUXCMD_DESKTOP_BUILD_UNIX").parse().unwrap_or(0)),
        tauri_version: tauri::VERSION.to_string(),
        signed: false,
        read_only: true,
    }
}

#[tauri::command]
pub async fn check_update(
    include_prerelease: Option<bool>,
    state: State<'_, Arc<AppState>>,
) -> Result<UpdateCheck, UpdateCheckError> {
    let include_prerelease = include_prerelease.unwrap_or(false);
    let state = Arc::clone(state.inner());
    let current = detect::detect_installation().suite_version;
    tauri::async_runtime::spawn_blocking(move || {
        let mut cache = state
            .cache
            .lock()
            .map_err(|_| UpdateCheckError::internal("内部缓存状态不可用"))?;
        release::check(&mut cache, include_prerelease, current.as_deref())
    })
    .await
    .map_err(|error| UpdateCheckError::internal(format!("检查任务失败：{error}")))?
}

#[tauri::command]
pub async fn verify_installation() -> Result<VerificationReport, String> {
    tauri::async_runtime::spawn_blocking(|| {
        let install = detect::detect_installation();
        integrity::verify(&install)
    })
    .await
    .map_err(|error| format!("校验任务失败：{error}"))
}

#[tauri::command]
pub fn open_external(url: String) -> Result<(), String> {
    let parsed = url::Url::parse(&url).map_err(|error| format!("无效链接：{error}"))?;
    if parsed.scheme() != "https" {
        return Err("只允许 https 链接".to_string());
    }
    let host = parsed.host_str().unwrap_or_default();
    if !ALLOWED_EXTERNAL_HOSTS.contains(&host) {
        return Err(format!("链接域名不在白名单内：{host}"));
    }
    std::process::Command::new("rundll32.exe")
        .arg("url.dll,FileProtocolHandler")
        .arg(parsed.as_str())
        .spawn()
        .map_err(|error| format!("无法打开系统浏览器：{error}"))?;
    Ok(())
}

#[tauri::command]
pub fn copy_diagnostics(app: tauri::AppHandle) -> Result<String, String> {
    let text = diagnostics_text();
    app.clipboard()
        .write_text(text.clone())
        .map_err(|error| format!("复制诊断信息失败：{error}"))?;
    Ok(text)
}

fn diagnostics_text() -> String {
    use std::fmt::Write as _;

    let install = detect::detect_installation();
    let manager = get_manager_info();
    let mut text = String::new();
    let _ = writeln!(text, "RuxCmd Desktop diagnostics");
    let _ = writeln!(text, "generated: {}", util::utc_now_iso());
    let _ = writeln!(
        text,
        "manager: {} ({}, built {}, tauri {}, signed={}, read-only={})",
        manager.version,
        manager.target,
        manager.build_time,
        manager.tauri_version,
        manager.signed,
        manager.read_only
    );
    let _ = writeln!(text, "install method: {:?}", install.method);
    let _ = writeln!(
        text,
        "suite version: {}",
        install.suite_version.as_deref().unwrap_or("unknown")
    );
    let _ = writeln!(
        text,
        "install dir: {}",
        install.install_dir.as_deref().unwrap_or("unknown")
    );
    let _ = writeln!(
        text,
        "source commit: {} (dirty={})",
        install.source_commit.as_deref().unwrap_or("unknown"),
        install
            .dirty
            .map(|dirty| dirty.to_string())
            .unwrap_or_else(|| "unknown".to_string())
    );
    let _ = writeln!(
        text,
        "target: {}",
        install.target.as_deref().unwrap_or("unknown")
    );
    let _ = writeln!(
        text,
        "rustc: {}",
        install.rustc_version.as_deref().unwrap_or("unknown")
    );
    let _ = writeln!(
        text,
        "binaries: {} (source: {})",
        install
            .binary_count
            .map(|count| count.to_string())
            .unwrap_or_else(|| "unknown".to_string()),
        install.counts_source.as_deref().unwrap_or("unknown")
    );
    let _ = writeln!(
        text,
        "shims: {}",
        install
            .shim_count
            .map(|count| count.to_string())
            .unwrap_or_else(|| "unknown".to_string())
    );
    let _ = writeln!(
        text,
        "user PATH: {}",
        match install.path_on_user_path {
            Some(true) => "contains install dir",
            Some(false) => "does not contain install dir",
            None => "unknown",
        }
    );
    let _ = writeln!(
        text,
        "checksum: not checked (run verify_installation)"
    );
    let _ = writeln!(
        text,
        "package sha256: {}",
        install.package_sha256.as_deref().unwrap_or("unknown")
    );
    let _ = writeln!(
        text,
        "installed at: {}",
        install.installed_at.as_deref().unwrap_or("unknown")
    );
    if install.conflicts.is_empty() {
        let _ = writeln!(text, "conflicts: none");
    } else {
        let _ = writeln!(text, "conflicts: {}", install.conflicts.join(" | "));
    }
    if !install.warnings.is_empty() {
        let _ = writeln!(text, "warnings: {}", install.warnings.join(" | "));
    }
    text
}
