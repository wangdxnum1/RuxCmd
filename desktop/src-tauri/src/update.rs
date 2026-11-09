//! P2 write path: prepare (download + verify), apply (hand off to a helper
//! that waits for this UI to exit), and cancel.
//!
//! The UI process never replaces its own running executable: `apply_update`
//! only writes `helper.ps1` and starts it detached (design §5.3 and §7).

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use tauri::{AppHandle, Emitter, State};

use crate::commands::AppState;
use crate::detect;
use crate::integrity;
use crate::model::{
    ApplyOutcome, InstallInfo, InstallMethod, PreparedUpdate, UpdateCheckError, UpdateFinished,
    UpdateKind, UpdateLog, UpdateProgress, UpdateStage,
};
use crate::release;

const TARGET: &str = release::TARGET;
const PROGRESS_STEP_BYTES: u64 = 512 * 1024;
/// `apply_update` waits this long for the UI to exit before giving up.
const UI_EXIT_TIMEOUT_SECONDS: u64 = 90;
/// Keeps the probe honest in tests without touching the real profile.
const UPDATE_ROOT_ENV: &str = "RUXCMD_DESKTOP_UPDATE_ROOT";

#[derive(Debug, Clone)]
pub struct PreparedInternal {
    pub update: PreparedUpdate,
    pub dir: PathBuf,
    pub artifact_path: PathBuf,
    pub install_dir: PathBuf,
    pub method: InstallMethod,
    pub current_version: Option<String>,
    pub handed_off: bool,
}

#[tauri::command]
pub async fn prepare_update(
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
    version: String,
    include_prerelease: Option<bool>,
) -> Result<PreparedUpdate, UpdateCheckError> {
    let state = Arc::clone(state.inner());
    let include_prerelease = include_prerelease.unwrap_or(false);
    tauri::async_runtime::spawn_blocking(move || {
        prepare_impl(&app, &state, &version, include_prerelease)
    })
    .await
    .map_err(|error| UpdateCheckError::internal(format!("准备更新任务失败：{error}")))?
}

#[tauri::command]
pub async fn apply_update(
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
    id: String,
    confirm_overwrite: Option<bool>,
) -> Result<ApplyOutcome, UpdateCheckError> {
    let state = Arc::clone(state.inner());
    let confirm_overwrite = confirm_overwrite.unwrap_or(false);
    tauri::async_runtime::spawn_blocking(move || apply_impl(&app, &state, &id, confirm_overwrite))
        .await
        .map_err(|error| UpdateCheckError::internal(format!("应用更新任务失败：{error}")))?
}

#[tauri::command]
pub fn cancel_update(
    state: State<'_, Arc<AppState>>,
    id: String,
) -> Result<bool, UpdateCheckError> {
    cancel_impl(state.inner(), &id)
}

#[tauri::command]
pub fn quit_for_update(
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
    id: String,
) -> Result<(), String> {
    let handed_off = state
        .prepared
        .lock()
        .map_err(|_| "内部状态不可用".to_string())?
        .as_ref()
        .filter(|prepared| prepared.update.id == id)
        .map(|prepared| prepared.handed_off)
        .unwrap_or(false);
    if !handed_off {
        return Err("更新还没有交给安装器，不能退出。".to_string());
    }
    app.exit(0);
    Ok(())
}

fn prepare_impl(
    app: &AppHandle,
    state: &AppState,
    requested: &str,
    include_prerelease: bool,
) -> Result<PreparedUpdate, UpdateCheckError> {
    let install = detect::detect_installation();
    let (kind, install_dir) = kind_for(&install)?;
    let current = install.suite_version.clone();

    emit_stage(app, "", "precheck");
    emit_log(
        app,
        "",
        format!(
            "目标版本 v{requested}；安装方式 {kind:?}；安装目录 {}",
            install_dir.display()
        ),
    );

    let check = {
        let mut cache = state
            .cache
            .lock()
            .map_err(|_| UpdateCheckError::internal("内部缓存状态不可用"))?;
        release::refresh(&mut cache, include_prerelease, current.as_deref())?
    };
    let latest = check
        .latest
        .as_ref()
        .ok_or_else(|| UpdateCheckError::invalid_response("返回的内容缺少发布对象"))?;

    let requested = requested.trim().trim_start_matches(&['v', 'V'][..]);
    if requested != latest.version {
        return Err(UpdateCheckError::invalid_response(format!(
            "发布版本已变化：请求 {requested}，当前最新 {}；请重新检查更新",
            latest.version
        )));
    }
    if !check.assets_complete {
        return Err(UpdateCheckError::invalid_response(format!(
            "发布资产不完整，缺少：{}",
            check.missing_assets.join("、")
        )));
    }

    let artifact = artifact_name(&latest.version, kind);
    let asset = latest
        .assets
        .iter()
        .find(|asset| asset.name == artifact && asset.state == "uploaded")
        .ok_or_else(|| {
            UpdateCheckError::invalid_response(format!("发布中找不到已上传的资产：{artifact}"))
        })?;
    let url = asset.download_url.clone().ok_or_else(|| {
        UpdateCheckError::invalid_response(format!("资产缺少下载地址：{artifact}"))
    })?;
    let sidecar = check
        .sidecar_checks
        .iter()
        .find(|entry| entry.binary == artifact)
        .ok_or_else(|| {
            UpdateCheckError::invalid_response(format!(
                "缺少 {artifact} 的 sidecar 校验结果，无法确认下载内容"
            ))
        })?;
    if sidecar.matches != Some(true) {
        let reason = sidecar
            .error
            .clone()
            .unwrap_or_else(|| "官方记录与 .sha256 sidecar 不一致".to_string());
        return Err(UpdateCheckError::invalid_response(format!(
            "{artifact} 未通过三方哈希校验：{reason}"
        )));
    }
    let sha256 = sidecar
        .sidecar_sha256
        .clone()
        .or_else(|| sidecar.digest_sha256.clone())
        .ok_or_else(|| UpdateCheckError::invalid_response("sidecar 未提供 SHA256"))?;

    let id = new_update_id();
    let dir = update_root()?.join(&id);
    std::fs::create_dir_all(&dir).map_err(|error| {
        UpdateCheckError::internal(format!("无法创建下载目录 {}：{error}", dir.display()))
    })?;
    let artifact_path = dir.join(&artifact);

    emit_stage(app, &id, "download");
    emit_log(
        app,
        &id,
        format!("开始下载 {artifact}（{} 字节）", asset.size),
    );
    let cancel = Arc::new(AtomicBool::new(false));
    if let Ok(mut slot) = state.cancel.lock() {
        *slot = Some((id.clone(), Arc::clone(&cancel)));
    }

    let mut last_emitted = 0u64;
    let downloaded =
        release::download(&url, &sha256, &artifact_path, &cancel, |received, total| {
            if received - last_emitted >= PROGRESS_STEP_BYTES || received == total {
                last_emitted = received;
                let _ = app.emit(
                    "update-progress",
                    UpdateProgress {
                        id: id.clone(),
                        stage: "download".to_string(),
                        received,
                        total,
                    },
                );
            }
        });
    clear_cancel(state, &id);

    let downloaded = match downloaded {
        Ok(hash) => hash,
        Err(error) => {
            if error.cancelled {
                emit_stage(app, &id, "cancelled");
            }
            let _ = std::fs::remove_dir_all(&dir);
            return Err(error);
        }
    };

    emit_stage(app, &id, "verify");
    emit_log(
        app,
        &id,
        format!("SHA256 校验通过：{downloaded}（与 sidecar 及官方记录一致）"),
    );

    let prepared = PreparedInternal {
        dir,
        artifact_path,
        install_dir,
        method: install.method,
        current_version: current,
        handed_off: false,
        update: PreparedUpdate {
            id: id.clone(),
            version: latest.version.clone(),
            kind,
            artifact: artifact.clone(),
            sha256,
            size: asset.size,
        },
    };
    let previous = {
        let mut slot = state
            .prepared
            .lock()
            .map_err(|_| UpdateCheckError::internal("内部更新状态不可用"))?;
        slot.replace(prepared.clone())
    };
    if let Some(previous) = previous
        && previous.dir != prepared.dir
    {
        let _ = std::fs::remove_dir_all(&previous.dir);
    }

    emit_stage(app, &id, "ready");
    emit_log(
        app,
        &id,
        format!("已就绪：v{}，等待确认后交给安装器", prepared.update.version),
    );
    Ok(prepared.update)
}

fn apply_impl(
    app: &AppHandle,
    state: &AppState,
    id: &str,
    confirm_overwrite: bool,
) -> Result<ApplyOutcome, UpdateCheckError> {
    let prepared = {
        let slot = state
            .prepared
            .lock()
            .map_err(|_| UpdateCheckError::internal("内部更新状态不可用"))?;
        slot.as_ref()
            .filter(|prepared| prepared.update.id == id)
            .cloned()
            .ok_or_else(|| UpdateCheckError::not_found("没有待安装的更新（可能已取消或已过期）"))?
    };
    if prepared.handed_off {
        return Err(UpdateCheckError::internal(
            "更新已交给安装器，不能重复执行。",
        ));
    }
    if !prepared.artifact_path.is_file() {
        return Err(UpdateCheckError::internal(format!(
            "下载文件已不存在：{}",
            prepared.artifact_path.display()
        )));
    }

    // G15: precheck the user's installation before overwriting anything.
    emit_stage(app, id, "precheck");
    let report = integrity::verify(&install_target(&prepared));
    let mut details = Vec::new();
    for file in &report.mismatched {
        details.push(format!("已修改：{file}"));
    }
    for file in &report.missing {
        details.push(format!("缺失：{file}"));
    }
    if report.status == crate::model::ChecksumStatus::NotChecked {
        emit_log(
            app,
            id,
            format!(
                "无法预检本地改动：{}；继续交给安装器",
                report.message.as_deref().unwrap_or("原因未知")
            ),
        );
    } else {
        emit_log(
            app,
            id,
            format!(
                "预检完成：{} 个文件与 share/SHA256SUMS 一致，{} 个被修改，{} 个缺失",
                report.checked,
                report.mismatched.len(),
                report.missing.len()
            ),
        );
    }
    if !details.is_empty() && !confirm_overwrite {
        return Err(UpdateCheckError::needs_confirmation(
            "以下文件被本地修改或缺失，继续更新会覆盖这些改动。确认后才能继续。",
            details,
        ));
    }

    let ui_pid = Some(std::process::id());
    let script = helper_script(&prepared, ui_pid);
    let helper_path = prepared.dir.join("helper.ps1");
    std::fs::write(&helper_path, script).map_err(|error| {
        UpdateCheckError::internal(format!(
            "无法写入更新助手 {}：{error}",
            helper_path.display()
        ))
    })?;

    emit_log(
        app,
        id,
        format!(
            "更新助手已写入 {}；等待 UI 退出后执行安装",
            helper_path.display()
        ),
    );
    spawn_helper(&helper_path)?;

    {
        let mut slot = state
            .prepared
            .lock()
            .map_err(|_| UpdateCheckError::internal("内部更新状态不可用"))?;
        if let Some(entry) = slot.as_mut()
            && entry.update.id == prepared.update.id
        {
            entry.handed_off = true;
        }
    }

    let rollback = rollback_path(&prepared);
    emit_stage(app, id, "handoff");
    if let Some(path) = &rollback {
        emit_log(app, id, format!("脚本安装的原生备份目录：{path}"));
    }
    let _ = app.emit(
        "update-finished",
        UpdateFinished {
            id: id.to_string(),
            outcome: "handedOff".to_string(),
            version: Some(prepared.update.version.clone()),
            rollback: rollback.clone(),
        },
    );

    Ok(ApplyOutcome {
        handed_off: true,
        kind: prepared.update.kind,
        version: prepared.update.version,
        helper: Some(helper_path.to_string_lossy().into_owned()),
    })
}

fn cancel_impl(state: &AppState, id: &str) -> Result<bool, UpdateCheckError> {
    let taken = {
        let mut slot = state
            .prepared
            .lock()
            .map_err(|_| UpdateCheckError::internal("内部更新状态不可用"))?;
        match slot.as_ref() {
            Some(prepared) if prepared.update.id == id => {
                if prepared.handed_off {
                    return Err(UpdateCheckError::internal(
                        "已交给安装器，不可取消；安装器会自行完成或回滚。",
                    ));
                }
                slot.take()
            }
            _ => None,
        }
    };
    if let Some(prepared) = taken {
        let _ = std::fs::remove_dir_all(&prepared.dir);
        return Ok(true);
    }

    let slot = state
        .cancel
        .lock()
        .map_err(|_| UpdateCheckError::internal("内部取消状态不可用"))?;
    match slot.as_ref() {
        Some((active, flag)) if active == id => {
            flag.store(true, Ordering::SeqCst);
            Ok(true)
        }
        _ => Err(UpdateCheckError::not_found("没有可取消的下载或更新")),
    }
}

fn clear_cancel(state: &AppState, id: &str) {
    if let Ok(mut slot) = state.cancel.lock()
        && slot.as_ref().is_some_and(|(active, _)| active == id)
    {
        *slot = None;
    }
}

fn install_target(prepared: &PreparedInternal) -> InstallInfo {
    InstallInfo {
        method: prepared.method,
        install_dir: Some(prepared.install_dir.to_string_lossy().into_owned()),
        ..InstallInfo::default()
    }
}

fn rollback_path(prepared: &PreparedInternal) -> Option<String> {
    if prepared.method != InstallMethod::Script {
        return None;
    }
    let current = prepared.current_version.as_deref()?;
    if current == prepared.update.version {
        return None;
    }
    let safe = current.replace(
        |character: char| {
            !character.is_ascii_alphanumeric()
                && character != '.'
                && character != '_'
                && character != '-'
                && character != '+'
        },
        "_",
    );
    Some(format!("{}.old-{safe}", prepared.install_dir.display()))
}

pub(crate) fn kind_for(install: &InstallInfo) -> Result<(UpdateKind, PathBuf), UpdateCheckError> {
    let dir = install
        .install_dir
        .as_deref()
        .map(PathBuf::from)
        .ok_or_else(|| {
            UpdateCheckError::unsupported("未检测到安装目录，无法应用内更新；请打开发布页手动安装")
        })?;
    match install.method {
        InstallMethod::Script => Ok((UpdateKind::Script, dir)),
        InstallMethod::Inno => Ok((UpdateKind::Inno, dir)),
        _ => Err(UpdateCheckError::unsupported(
            "仅脚本安装 (install.ps1) 与 Inno Setup 安装支持应用内更新；便携解压或未安装请打开发布页手动更新",
        )),
    }
}

pub(crate) fn artifact_name(version: &str, kind: UpdateKind) -> String {
    match kind {
        UpdateKind::Script => format!("ruxcmd-{version}-{TARGET}.zip"),
        UpdateKind::Inno => format!("ruxcmd-{version}-{TARGET}-setup.exe"),
    }
}

fn new_update_id() -> String {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|elapsed| elapsed.as_nanos())
        .unwrap_or(0);
    format!("{nanos:x}-{:x}", std::process::id())
}

fn update_root() -> Result<PathBuf, UpdateCheckError> {
    if let Some(value) = std::env::var_os(UPDATE_ROOT_ENV) {
        let path = PathBuf::from(value);
        if !path.as_os_str().is_empty() {
            return Ok(path);
        }
    }
    let local = std::env::var_os("LOCALAPPDATA")
        .ok_or_else(|| UpdateCheckError::internal("无法解析 %LOCALAPPDATA%，不能确定下载目录"))?;
    Ok(PathBuf::from(local)
        .join("RuxCmd")
        .join("Desktop")
        .join("updates"))
}

/// Escapes a value for a PowerShell single-quoted string.
fn ps_literal(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}

/// Builds the detached helper. It waits for the UI to exit, then hands the
/// verified artifact to the native installer and reports into `helper.log`.
fn helper_script(prepared: &PreparedInternal, ui_pid: Option<u32>) -> String {
    let mut lines: Vec<String> = Vec::new();
    let pid = ui_pid.unwrap_or(0);

    lines.push("#Requires -Version 5.1".to_string());
    lines.push(format!(
        "# Generated by RuxCmd Desktop {} for version {}.",
        env!("RUXCMD_DESKTOP_VERSION"),
        prepared.update.version
    ));
    lines.push("$ErrorActionPreference = 'Stop'".to_string());
    lines.push(format!(
        "$LogPath = {}",
        ps_literal(&prepared.dir.join("helper.log").to_string_lossy())
    ));
    lines.push("function Write-Log {".to_string());
    lines.push("    param([string]$Line)".to_string());
    lines.push(
        "    Add-Content -LiteralPath $LogPath -Value ('[' + (Get-Date -Format o) + '] ' + $Line)"
            .to_string(),
    );
    lines.push("}".to_string());
    lines.push(format!("$UiPid = {pid}"));
    lines.push("Write-Log 'RuxCmd Desktop update helper started'".to_string());
    lines.push("if ($UiPid -gt 0) {".to_string());
    lines.push(format!(
        "    $Deadline = (Get-Date).AddSeconds({UI_EXIT_TIMEOUT_SECONDS})"
    ));
    lines.push("    while ((Get-Date) -lt $Deadline) {".to_string());
    lines.push(
        "        $Running = Get-Process -Id $UiPid -ErrorAction SilentlyContinue".to_string(),
    );
    lines.push("        if (-not $Running) { break }".to_string());
    lines.push("        Start-Sleep -Milliseconds 500".to_string());
    lines.push("    }".to_string());
    lines.push("    if (Get-Process -Id $UiPid -ErrorAction SilentlyContinue) {".to_string());
    lines.push(format!(
        "        Write-Log 'The desktop UI is still running after {UI_EXIT_TIMEOUT_SECONDS} seconds; aborting without changes.'"
    ));
    lines.push("        exit 2".to_string());
    lines.push("    }".to_string());
    lines.push("    Write-Log 'The desktop UI has exited; continuing.'".to_string());
    lines.push("}".to_string());

    match prepared.update.kind {
        UpdateKind::Script => {
            let installer = prepared.install_dir.join("share").join("install.ps1");
            lines.push(format!(
                "$Installer = {}",
                ps_literal(&installer.to_string_lossy())
            ));
            lines.push(format!(
                "$Archive = {}",
                ps_literal(&prepared.artifact_path.to_string_lossy())
            ));
            lines.push(format!("$Sha256 = {}", ps_literal(&prepared.update.sha256)));
            lines.push(format!(
                "$InstallDir = {}",
                ps_literal(&prepared.install_dir.to_string_lossy())
            ));
            lines.push(
                "if (-not (Test-Path -LiteralPath $Installer -PathType Leaf)) { Write-Log \"install.ps1 not found: $Installer\"; exit 3 }"
                    .to_string(),
            );
            lines.push("$Pwsh = Get-Command pwsh -ErrorAction SilentlyContinue".to_string());
            lines.push(
                "if ($null -eq $Pwsh) { Write-Log 'pwsh (PowerShell 7) was not found on PATH; install.ps1 requires it.'; exit 3 }"
                    .to_string(),
            );
            lines.push("Write-Log \"Running install.ps1 with $Archive\"".to_string());
            lines.push(
                "& $Pwsh.Source -NoProfile -ExecutionPolicy Bypass -File $Installer -Source $Archive -Sha256 $Sha256 -InstallDir $InstallDir -Force 2>&1 | ForEach-Object { Write-Log ([string]$_) }"
                    .to_string(),
            );
            lines.push(
                "if ($LASTEXITCODE -ne 0) { Write-Log \"install.ps1 failed with exit code $LASTEXITCODE\"; exit 4 }"
                    .to_string(),
            );
        }
        UpdateKind::Inno => {
            let setup = prepared.artifact_path.clone();
            let setup_log = prepared.dir.join("setup.log");
            lines.push(format!("$Setup = {}", ps_literal(&setup.to_string_lossy())));
            lines.push(
                "if (-not (Test-Path -LiteralPath $Setup -PathType Leaf)) { Write-Log \"setup.exe not found: $Setup\"; exit 3 }"
                    .to_string(),
            );
            lines.push("$Arguments = @('/VERYSILENT', '/SUPPRESSMSGBOXES', '/NORESTART', '/CLOSEAPPLICATIONS')".to_string());
            lines.push(format!(
                "$Arguments += {}",
                ps_literal(&format!("/LOG=\"{}\"", setup_log.display()))
            ));
            lines.push(
                "Write-Log 'Running setup.exe silently; approve the UAC prompt if Windows shows one.'"
                    .to_string(),
            );
            lines.push(
                "$Process = Start-Process -FilePath $Setup -ArgumentList $Arguments -Wait -PassThru"
                    .to_string(),
            );
            lines.push(
                "if ($Process.ExitCode -ne 0) { Write-Log \"setup.exe failed with exit code $($Process.ExitCode)\"; exit 4 }"
                    .to_string(),
            );
        }
    }

    // G13 B: the UI was running (it started this helper) and the install has
    // just succeeded, so relaunch it.
    lines.push("if ($UiPid -gt 0) {".to_string());
    lines.push(format!(
        "    $Desktop = {}",
        ps_literal(
            &prepared
                .install_dir
                .join("ruxcmd-desktop.exe")
                .to_string_lossy()
        )
    ));
    lines.push("    if (Test-Path -LiteralPath $Desktop -PathType Leaf) {".to_string());
    lines.push("        Write-Log 'Restarting RuxCmd Desktop'".to_string());
    lines.push("        Start-Process -FilePath $Desktop".to_string());
    lines.push("    } else {".to_string());
    lines.push(
        "        Write-Log \"ruxcmd-desktop.exe not found after the update: $Desktop\"".to_string(),
    );
    lines.push("    }".to_string());
    lines.push("}".to_string());
    lines.push("Write-Log 'Update helper finished.'".to_string());
    lines.push("exit 0".to_string());
    lines.join("\r\n") + "\r\n"
}

fn spawn_helper(helper: &Path) -> Result<(), UpdateCheckError> {
    use std::os::windows::process::CommandExt;

    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    let working_dir = helper.parent().unwrap_or(Path::new("."));
    let mut command = std::process::Command::new("powershell.exe");
    command
        .arg("-NoProfile")
        .arg("-ExecutionPolicy")
        .arg("Bypass")
        .arg("-File")
        .arg(helper)
        .current_dir(working_dir)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .creation_flags(CREATE_NO_WINDOW);
    command
        .spawn()
        .map(|_| ())
        .map_err(|error| UpdateCheckError::internal(format!("无法启动更新助手：{error}")))
}

fn emit_stage(app: &AppHandle, id: &str, stage: &str) {
    let _ = app.emit(
        "update-stage",
        UpdateStage {
            id: id.to_string(),
            stage: stage.to_string(),
        },
    );
}

fn emit_log(app: &AppHandle, id: &str, line: impl Into<String>) {
    let _ = app.emit(
        "update-log",
        UpdateLog {
            id: id.to_string(),
            line: line.into(),
        },
    );
}

#[cfg(test)]
mod tests {
    use super::{
        PreparedInternal, artifact_name, helper_script, kind_for, new_update_id, ps_literal,
        rollback_path,
    };
    use crate::model::{InstallInfo, InstallMethod, PreparedUpdate, UpdateKind};
    use sha2::{Digest, Sha256};
    use std::path::PathBuf;

    fn install(method: InstallMethod, dir: Option<&str>) -> InstallInfo {
        InstallInfo {
            method,
            install_dir: dir.map(str::to_string),
            ..InstallInfo::default()
        }
    }

    fn prepared(kind: UpdateKind, dir: &str, current: Option<&str>) -> PreparedInternal {
        let version = "0.6.0".to_string();
        let artifact = artifact_name(&version, kind);
        PreparedInternal {
            update: PreparedUpdate {
                id: "test".to_string(),
                version,
                kind,
                artifact: artifact.clone(),
                sha256: "a".repeat(64),
                size: 1024,
            },
            dir: PathBuf::from(r"C:\updates\test"),
            artifact_path: PathBuf::from(r"C:\updates\test").join(artifact),
            install_dir: PathBuf::from(dir),
            method: if kind == UpdateKind::Script {
                InstallMethod::Script
            } else {
                InstallMethod::Inno
            },
            current_version: current.map(str::to_string),
            handed_off: false,
        }
    }

    #[test]
    fn maps_install_methods_to_update_kinds() {
        let (kind, dir) = kind_for(&install(InstallMethod::Script, Some(r"C:\RuxCmd\bin")))
            .expect("script install");
        assert_eq!(kind, UpdateKind::Script);
        assert_eq!(dir, PathBuf::from(r"C:\RuxCmd\bin"));

        let (kind, _) =
            kind_for(&install(InstallMethod::Inno, Some(r"C:\RuxCmd\bin"))).expect("inno install");
        assert_eq!(kind, UpdateKind::Inno);

        for method in [
            InstallMethod::Portable,
            InstallMethod::NotInstalled,
            InstallMethod::Unknown,
        ] {
            let error =
                kind_for(&install(method, Some(r"C:\RuxCmd"))).expect_err("no in-app update");
            assert_eq!(error.kind, "unsupported");
        }
        assert_eq!(
            kind_for(&install(InstallMethod::Script, None))
                .expect_err("no dir")
                .kind,
            "unsupported"
        );
    }

    #[test]
    fn builds_release_artifact_names() {
        assert_eq!(
            artifact_name("0.6.0", UpdateKind::Script),
            "ruxcmd-0.6.0-x86_64-pc-windows-msvc.zip"
        );
        assert_eq!(
            artifact_name("0.6.0", UpdateKind::Inno),
            "ruxcmd-0.6.0-x86_64-pc-windows-msvc-setup.exe"
        );
    }

    #[test]
    fn escapes_powershell_literals() {
        assert_eq!(ps_literal(r"C:\RuxCmd\bin"), r"'C:\RuxCmd\bin'");
        assert_eq!(
            ps_literal(r"C:\Users\O'Brien\RuxCmd"),
            r"'C:\Users\O''Brien\RuxCmd'"
        );
    }

    #[test]
    fn script_helper_waits_for_the_ui_and_runs_install_ps1() {
        let prepared = prepared(UpdateKind::Script, r"C:\Apps\RuxCmd\bin", Some("0.5.11"));
        let script = helper_script(&prepared, Some(4242));
        assert!(script.contains("$UiPid = 4242"));
        assert!(script.contains("AddSeconds(90)"));
        assert!(script.contains(r"'C:\Apps\RuxCmd\bin\share\install.ps1'"));
        assert!(script.contains(r"ruxcmd-0.6.0-x86_64-pc-windows-msvc.zip"));
        assert!(script.contains("-Force"));
        assert!(script.contains(r"'C:\Apps\RuxCmd\bin'"));
        assert!(script.contains(r"ruxcmd-desktop.exe"));
        assert!(!script.contains("setup.exe"));
    }

    #[test]
    fn inno_helper_runs_the_silent_setup() {
        let prepared = prepared(UpdateKind::Inno, r"C:\Apps\RuxCmd\bin", Some("0.5.11"));
        let script = helper_script(&prepared, None);
        assert!(script.contains("$UiPid = 0"));
        assert!(script.contains(r"ruxcmd-0.6.0-x86_64-pc-windows-msvc-setup.exe"));
        assert!(script.contains("/VERYSILENT"));
        assert!(script.contains("/SUPPRESSMSGBOXES"));
        assert!(script.contains("/LOG="));
        assert!(script.contains("Start-Process"));
        assert!(!script.contains("install.ps1"));
        assert!(
            script.contains("if ($UiPid -gt 0) {"),
            "restart stays conditional"
        );
    }

    #[test]
    fn reports_the_script_backup_directory() {
        let script = prepared(UpdateKind::Script, r"C:\Apps\RuxCmd\bin", Some("0.5.11"));
        assert_eq!(
            rollback_path(&script).as_deref(),
            Some(r"C:\Apps\RuxCmd\bin.old-0.5.11")
        );
        let inno = prepared(UpdateKind::Inno, r"C:\Apps\RuxCmd\bin", Some("0.5.11"));
        assert!(rollback_path(&inno).is_none());
    }

    /// End-to-end upgrade rehearsal. It runs the real `install.ps1` twice in a
    /// temporary directory and points the PATH seam at a file, so the real
    /// user PATH and the registry are never touched (design §9, G18).
    ///
    /// Ignored by default; run manually with:
    /// `cargo test --locked rehearsal_script_self_update -- --ignored --nocapture`
    #[test]
    #[ignore = "runs real PowerShell installers in a temporary directory"]
    fn rehearsal_script_self_update() {
        use std::process::Command;

        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let scripts = manifest_dir.join("..").join("..").join("scripts");
        assert!(
            scripts.join("install.ps1").is_file(),
            "install.ps1 not found under {}",
            scripts.display()
        );
        assert!(
            Command::new("pwsh")
                .args(["-NoProfile", "-NonInteractive", "-Command", "exit 0"])
                .status()
                .map(|status| status.success())
                .unwrap_or(false),
            "pwsh 7 is required for install.ps1"
        );

        let root =
            std::env::temp_dir().join(format!("ruxcmd-update-rehearsal-{}", new_update_id()));
        std::fs::create_dir_all(&root).expect("create temp root");
        let _guard = TempRoot(root.clone());

        let path_file = root.join("user-path.txt");
        let install_dir = root.join("installed").join("bin");
        let (first_zip, first_sha) = build_package(&root, &scripts, "0.1.0", "first");
        let (second_zip, second_sha) = build_package(&root, &scripts, "0.2.0", "second");

        // 1. Real first install through install.ps1.
        let status = Command::new("pwsh")
            .args([
                "-NoProfile",
                "-NonInteractive",
                "-ExecutionPolicy",
                "Bypass",
                "-File",
            ])
            .arg(scripts.join("install.ps1"))
            .args(["-Source"])
            .arg(&first_zip)
            .args(["-Sha256"])
            .arg(&first_sha)
            .args(["-InstallDir"])
            .arg(&install_dir)
            .arg("-Force")
            .env("RUXCMD_INSTALL_USER_PATH_FILE", &path_file)
            .status()
            .expect("run install.ps1");
        assert!(status.success(), "first install failed");
        assert_eq!(read_state_version(&install_dir), "0.1.0");
        assert_eq!(
            std::fs::read_to_string(install_dir.join("find.exe")).unwrap(),
            "find-0.1.0-first"
        );
        assert!(
            install_dir.join("share").join("install.ps1").is_file(),
            "the ZIP must ship install.ps1 so later upgrades can find it"
        );

        // 2. Build the same helper the UI would hand off.
        let update_dir = root.join("updates").join("rehearsal");
        std::fs::create_dir_all(&update_dir).expect("update dir");
        let prepared = PreparedInternal {
            update: PreparedUpdate {
                id: "rehearsal".to_string(),
                version: "0.2.0".to_string(),
                kind: UpdateKind::Script,
                artifact: second_zip
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
                    .into_owned(),
                sha256: second_sha.clone(),
                size: std::fs::metadata(&second_zip).unwrap().len(),
            },
            dir: update_dir.clone(),
            artifact_path: second_zip.clone(),
            install_dir: install_dir.clone(),
            method: InstallMethod::Script,
            current_version: Some("0.1.0".to_string()),
            handed_off: false,
        };
        let helper = update_dir.join("helper.ps1");
        std::fs::write(&helper, helper_script(&prepared, None)).expect("write helper");

        // 3. Run the helper; it must wait (no UI here), install, and exit 0.
        let status = Command::new("powershell.exe")
            .args(["-NoProfile", "-ExecutionPolicy", "Bypass", "-File"])
            .arg(&helper)
            .env("RUXCMD_INSTALL_USER_PATH_FILE", &path_file)
            .status()
            .expect("run helper.ps1");
        let helper_log = std::fs::read_to_string(update_dir.join("helper.log")).unwrap_or_default();
        assert!(
            status.success(),
            "helper failed ({}):\n{helper_log}",
            status.code().unwrap_or(-1)
        );

        // 4. The upgrade really replaced the files and kept a native backup.
        assert_eq!(read_state_version(&install_dir), "0.2.0");
        assert_eq!(
            std::fs::read_to_string(install_dir.join("find.exe")).unwrap(),
            "find-0.2.0-second"
        );
        assert_eq!(
            std::fs::read_to_string(install_dir.join("share").join("README.md")).unwrap(),
            "readme-0.2.0-second"
        );
        let backup = PathBuf::from(format!("{}.old-0.1.0", install_dir.display()));
        assert!(backup.join("find.exe").is_file(), "backup missing");
        assert!(helper_log.contains("Update helper finished."));
        let path_text = std::fs::read_to_string(&path_file).unwrap_or_default();
        assert!(
            path_text.contains("installed"),
            "user PATH seam was updated: {path_text}"
        );
    }

    struct TempRoot(PathBuf);

    impl Drop for TempRoot {
        fn drop(&mut self) {
            // Defender and the installer may still hold a handle briefly.
            for _ in 0..8 {
                if std::fs::remove_dir_all(&self.0).is_ok() {
                    return;
                }
                std::thread::sleep(std::time::Duration::from_millis(150));
            }
        }
    }

    fn read_state_version(install_dir: &std::path::Path) -> String {
        let text = std::fs::read_to_string(install_dir.join(".ruxcmd-install.json"))
            .expect("read install state");
        let state: serde_json::Value = serde_json::from_str(&text).expect("parse install state");
        state["version"].as_str().unwrap_or_default().to_string()
    }

    fn build_package(
        root: &std::path::Path,
        scripts: &std::path::Path,
        version: &str,
        marker: &str,
    ) -> (PathBuf, String) {
        use std::process::Command;

        let stage = root.join(format!("stage-{version}"));
        std::fs::create_dir_all(stage.join("bin")).expect("stage bin");
        std::fs::write(
            stage.join("bin").join("find.exe"),
            format!("find-{version}-{marker}"),
        )
        .expect("stage exe");
        std::fs::write(
            stage.join("README.md"),
            format!("readme-{version}-{marker}"),
        )
        .expect("stage readme");
        std::fs::copy(scripts.join("install.ps1"), stage.join("install.ps1"))
            .expect("stage install.ps1");
        std::fs::copy(scripts.join("common.ps1"), stage.join("common.ps1"))
            .expect("stage common.ps1");
        std::fs::copy(
            scripts.join("process-job.ps1"),
            stage.join("process-job.ps1"),
        )
        .expect("stage process-job.ps1");
        std::fs::write(
            stage.join("manifest.json"),
            format!("{{\"suiteVersion\":\"{version}\"}}"),
        )
        .expect("stage manifest");

        let mut sums = String::new();
        for entry in [
            "bin/find.exe",
            "README.md",
            "install.ps1",
            "common.ps1",
            "process-job.ps1",
        ] {
            let path = stage.join(entry.replace('/', std::path::MAIN_SEPARATOR_STR));
            sums.push_str(&format!("{}  {entry}\n", sha256_file(&path)));
        }
        std::fs::write(stage.join("SHA256SUMS"), sums).expect("stage sums");

        let zip = root.join(format!("ruxcmd-{version}-x86_64-pc-windows-msvc.zip"));
        let command = format!(
            "Add-Type -AssemblyName System.IO.Compression.FileSystem; [IO.Compression.ZipFile]::CreateFromDirectory({}, {})",
            ps_literal(&stage.to_string_lossy()),
            ps_literal(&zip.to_string_lossy())
        );
        let status = Command::new("pwsh")
            .args(["-NoProfile", "-NonInteractive", "-Command", &command])
            .status()
            .expect("run Compress-Archive");
        assert!(status.success(), "failed to create {}", zip.display());
        let hash = sha256_file(&zip);
        (zip, hash)
    }

    fn sha256_file(path: &std::path::Path) -> String {
        use std::io::Read as _;

        let mut file = std::fs::File::open(path).expect("open for hashing");
        let mut hasher = Sha256::new();
        let mut buffer = [0u8; 8192];
        loop {
            let read = file.read(&mut buffer).expect("hash read");
            if read == 0 {
                break;
            }
            hasher.update(&buffer[..read]);
        }
        format!("{:x}", hasher.finalize())
    }
}
