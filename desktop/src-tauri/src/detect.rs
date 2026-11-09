use std::path::{Path, PathBuf};

use serde::Deserialize;
use winreg::HKEY;
use winreg::RegKey;
use winreg::enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, KEY_READ};

use crate::model::{ChecksumStatus, InstallInfo, InstallMethod, InstalledAtKind};

const UNINSTALL_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Uninstall\{06BD23F3-4290-4F15-AEC6-55FD9DC91B9F}_is1";
const USER_ENV_KEY: &str = "Environment";
const STATE_FILE: &str = ".ruxcmd-install.json";

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ScriptState {
    version: Option<String>,
    target: Option<String>,
    sha256: Option<String>,
    installed_at: Option<String>,
    install_dir: Option<String>,
    #[serde(default)]
    binaries: Vec<String>,
    #[serde(default)]
    shims: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SuiteManifest {
    suite_version: Option<String>,
    target: Option<String>,
    rustc_version: Option<String>,
    source_commit: Option<String>,
    dirty: Option<bool>,
    binaries: Option<Vec<serde_json::Value>>,
}

#[derive(Debug, Clone)]
struct Candidate {
    dir: PathBuf,
    state: Option<ScriptState>,
    manifest: Option<SuiteManifest>,
    manifest_problem: Option<String>,
}

#[derive(Debug, Clone)]
struct InnoRecord {
    version: Option<String>,
    dir: Option<PathBuf>,
    /// Day-only `YYYYMMDD` from the uninstall key's `InstallDate` value.
    install_date: Option<String>,
}

/// Read-only installation probe. Never mutates the file system or the registry.
pub fn detect_installation() -> InstallInfo {
    let hkcu = read_inno(HKEY_CURRENT_USER);
    let hklm = read_inno(HKEY_LOCAL_MACHINE);
    let inno = hkcu.clone().or(hklm.clone());

    let mut conflicts = Vec::new();
    let mut warnings = Vec::new();

    if let (Some(user), Some(machine)) = (&hkcu, &hklm) {
        if user.version != machine.version {
            conflicts.push(format!(
                "HKCU 卸载记录版本 {} 与 HKLM 卸载记录版本 {} 不一致",
                user.version.as_deref().unwrap_or("未知"),
                machine.version.as_deref().unwrap_or("未知")
            ));
        }
        if !same_path(user.dir.as_deref(), machine.dir.as_deref()) {
            conflicts.push("HKCU 与 HKLM 卸载记录的安装目录不一致".to_string());
        }
    }

    let mut candidates = Vec::new();
    if let Some(dir) = hkcu.as_ref().and_then(|record| record.dir.clone()) {
        add_candidate(&mut candidates, dir);
    }
    if let Some(dir) = hklm.as_ref().and_then(|record| record.dir.clone()) {
        add_candidate(&mut candidates, dir);
    }
    if let Some(dir) = default_install_dir() {
        add_candidate(&mut candidates, dir);
    }

    let script = candidates
        .iter()
        .find(|candidate| candidate.state.is_some());
    let manifest_candidate = candidates
        .iter()
        .find(|candidate| candidate.manifest.is_some());

    let (method, install_dir) = if let Some(record) = &inno {
        if script.is_some() {
            conflicts.push("脚本安装状态文件与 Inno 卸载记录同时存在".to_string());
        }
        (InstallMethod::Inno, record.dir.clone())
    } else if let Some(candidate) = script {
        let dir = candidate
            .state
            .as_ref()
            .and_then(|state| state.install_dir.as_deref())
            .map(trimmed_dir)
            .unwrap_or_else(|| candidate.dir.clone());
        (InstallMethod::Script, Some(dir))
    } else if let Some(candidate) = manifest_candidate {
        (InstallMethod::Portable, Some(candidate.dir.clone()))
    } else if candidates
        .iter()
        .any(|candidate| has_executables(&candidate.dir))
    {
        warnings.push("安装目录存在可执行文件，但缺少状态文件与发布清单".to_string());
        (
            InstallMethod::Unknown,
            candidates.first().map(|candidate| candidate.dir.clone()),
        )
    } else {
        (InstallMethod::NotInstalled, None)
    };

    let candidate = install_dir
        .as_ref()
        .map(|dir| read_candidate(dir))
        .or_else(|| candidates.first().cloned());

    let state = candidate
        .as_ref()
        .and_then(|candidate| candidate.state.as_ref());
    let manifest = candidate
        .as_ref()
        .and_then(|candidate| candidate.manifest.as_ref());
    if let Some(problem) = candidate
        .as_ref()
        .and_then(|candidate| candidate.manifest_problem.as_ref())
    {
        warnings.push(problem.clone());
    }

    let registry_version = inno.as_ref().and_then(|record| record.version.clone());
    let state_version = state.and_then(|state| state.version.clone());
    let manifest_version = manifest.and_then(|manifest| manifest.suite_version.clone());

    let suite_version = match method {
        InstallMethod::Inno => registry_version.clone(),
        InstallMethod::Script => state_version.clone(),
        _ => manifest_version.clone(),
    };

    if let (Some(registry), Some(published)) = (&registry_version, &manifest_version)
        && registry != published
    {
        conflicts.push(format!(
            "卸载记录版本 {registry} 与 share/manifest.json 版本 {published} 不一致"
        ));
    }
    if let (Some(installed), Some(published)) = (&state_version, &manifest_version)
        && installed != published
    {
        conflicts.push(format!(
            "状态文件版本 {installed} 与 share/manifest.json 版本 {published} 不一致"
        ));
    }

    if method != InstallMethod::NotInstalled && manifest.is_none() && warnings.is_empty() {
        warnings.push("未找到 share/manifest.json，无法显示发布溯源".to_string());
    }

    let (binary_count, shim_count, counts_source) = counts(state, manifest, install_dir.as_deref());

    // The script installer records a full UTC timestamp; Inno only records the
    // install day in the uninstall key. Fall back to that day for Inno installs
    // so the About view is not left with an empty "安装时间" row.
    let (installed_at, installed_at_kind) = match state.and_then(|state| state.installed_at.clone())
    {
        Some(timestamp) => (Some(timestamp), InstalledAtKind::Timestamp),
        None => match method {
            InstallMethod::Inno => (
                inno.as_ref().and_then(|record| record.install_date.clone()),
                InstalledAtKind::Date,
            ),
            _ => (None, InstalledAtKind::Unknown),
        },
    };

    let path_on_user_path = match install_dir.as_deref() {
        Some(dir) => user_path_contains(dir),
        None => default_install_dir()
            .as_deref()
            .and_then(user_path_contains),
    };
    if method == InstallMethod::NotInstalled
        && path_on_user_path == Some(true)
        && let Some(dir) = default_install_dir()
    {
        warnings.push(format!(
            "用户 PATH 仍包含 {}，但未找到安装记录",
            dir.display()
        ));
    }

    InstallInfo {
        method,
        suite_version,
        install_dir: install_dir.map(|dir| dir.to_string_lossy().into_owned()),
        source_commit: manifest.and_then(|manifest| manifest.source_commit.clone()),
        dirty: manifest.and_then(|manifest| manifest.dirty),
        target: manifest
            .and_then(|manifest| manifest.target.clone())
            .or_else(|| state.and_then(|state| state.target.clone())),
        rustc_version: manifest.and_then(|manifest| manifest.rustc_version.clone()),
        binary_count,
        shim_count,
        counts_source,
        checksum_status: ChecksumStatus::NotChecked,
        path_on_user_path,
        package_sha256: state.and_then(|state| state.sha256.clone()),
        installed_at,
        installed_at_kind,
        conflicts,
        warnings,
    }
}

fn read_inno(root: HKEY) -> Option<InnoRecord> {
    let key = RegKey::predef(root)
        .open_subkey_with_flags(UNINSTALL_KEY, KEY_READ)
        .ok()?;
    let version = key.get_value::<String, _>("DisplayVersion").ok();
    let dir = key
        .get_value::<String, _>("InstallLocation")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .map(|value| trimmed_dir(&value));
    // Inno records the install day (not a time). Accept both the string form
    // used by Inno 7 and the DWORD form older installers may leave behind.
    let install_date = match key.get_value::<String, _>("InstallDate") {
        Ok(value) => normalize_install_date(&value),
        Err(_) => key
            .get_value::<u32, _>("InstallDate")
            .ok()
            .map(|value| value.to_string())
            .and_then(|value| normalize_install_date(&value)),
    };
    if version.is_none() && dir.is_none() {
        return None;
    }
    Some(InnoRecord {
        version,
        dir,
        install_date,
    })
}

/// Keep only a well-formed `YYYYMMDD` day, so the UI never renders a malformed
/// registry value as if it were a date.
fn normalize_install_date(value: &str) -> Option<String> {
    let text = value.trim();
    if text.len() != 8 || !text.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    let year: u32 = text[0..4].parse().ok()?;
    let month: u32 = text[4..6].parse().ok()?;
    let day: u32 = text[6..8].parse().ok()?;
    if !(2000..=2100).contains(&year) || !(1..=12).contains(&month) || !(1..=31).contains(&day) {
        return None;
    }
    Some(text.to_string())
}

fn add_candidate(candidates: &mut Vec<Candidate>, dir: PathBuf) {
    if candidates
        .iter()
        .any(|candidate| same_path(Some(&candidate.dir), Some(&dir)))
    {
        return;
    }
    candidates.push(read_candidate(&dir));
}

fn read_candidate(dir: &Path) -> Candidate {
    let state = std::fs::read_to_string(dir.join(STATE_FILE))
        .ok()
        .and_then(|text| serde_json::from_str::<ScriptState>(&text).ok());
    let manifest_path = dir.join("share").join("manifest.json");
    let (manifest, manifest_problem) = match std::fs::read_to_string(&manifest_path) {
        Ok(text) => match serde_json::from_str::<SuiteManifest>(&text) {
            Ok(manifest) => (Some(manifest), None),
            Err(error) => (None, Some(format!("share/manifest.json 无法解析：{error}"))),
        },
        Err(_) => (None, None),
    };
    Candidate {
        dir: dir.to_path_buf(),
        state,
        manifest,
        manifest_problem,
    }
}

fn default_install_dir() -> Option<PathBuf> {
    std::env::var_os("LOCALAPPDATA").map(|value| PathBuf::from(value).join("RuxCmd").join("bin"))
}

fn counts(
    state: Option<&ScriptState>,
    manifest: Option<&SuiteManifest>,
    dir: Option<&Path>,
) -> (Option<usize>, Option<usize>, Option<String>) {
    let mut sources = Vec::new();
    let mut binaries = None;
    let mut shims = None;

    if let Some(state) = state {
        if !state.binaries.is_empty() {
            binaries = Some(state.binaries.len());
            sources.push("state-file");
        }
        if !state.shims.is_empty() {
            shims = Some(state.shims.len());
        }
    }
    if binaries.is_none()
        && let Some(manifest) = manifest
        && let Some(list) = manifest.binaries.as_ref()
        && !list.is_empty()
    {
        binaries = Some(list.len());
        sources.push("manifest");
    }
    if binaries.is_none()
        && let Some(dir) = dir
        && dir.is_dir()
    {
        let count = std::fs::read_dir(dir)
            .map(|entries| {
                entries
                    .flatten()
                    .filter(|entry| {
                        let name = entry.file_name().to_string_lossy().to_lowercase();
                        name.ends_with(".exe") && !name.starts_with("unins")
                    })
                    .count()
            })
            .unwrap_or(0);
        binaries = Some(count);
        sources.push("directory-scan");
    }
    if shims.is_none()
        && let Some(dir) = dir
    {
        let count = std::fs::read_dir(dir)
            .map(|entries| {
                entries
                    .flatten()
                    .filter(|entry| {
                        let name = entry.file_name().to_string_lossy().to_lowercase();
                        name.starts_with("rux-") && name.ends_with(".cmd")
                    })
                    .count()
            })
            .unwrap_or(0);
        shims = Some(count);
    }

    let source = if sources.is_empty() {
        None
    } else {
        Some(sources.join(" + "))
    };
    (binaries, shims, source)
}

fn user_path_contains(dir: &Path) -> Option<bool> {
    let key = RegKey::predef(HKEY_CURRENT_USER)
        .open_subkey_with_flags(USER_ENV_KEY, KEY_READ)
        .ok()?;
    let value = key.get_value::<String, _>("Path").unwrap_or_default();
    let target = normalize_entry(&dir.to_string_lossy());
    Some(
        value
            .split(';')
            .filter(|entry| !entry.trim().is_empty())
            .any(|entry| normalize_entry(entry) == target),
    )
}

fn normalize_entry(value: &str) -> String {
    let expanded = expand_percent(value);
    let trimmed = expanded.trim().trim_matches('"');
    let mut normalized = trimmed.to_string();
    while normalized.len() > 3 && (normalized.ends_with('\\') || normalized.ends_with('/')) {
        normalized.pop();
    }
    normalized.to_lowercase()
}

fn expand_percent(value: &str) -> String {
    let mut output = String::new();
    let mut rest = value;
    while let Some(start) = rest.find('%') {
        output.push_str(&rest[..start]);
        let after = &rest[start + 1..];
        match after.find('%') {
            Some(end) => {
                let name = &after[..end];
                match std::env::var(name).or_else(|_| std::env::var(name.to_uppercase())) {
                    Ok(replacement) => output.push_str(&replacement),
                    Err(_) => {
                        output.push('%');
                        output.push_str(name);
                        output.push('%');
                    }
                }
                rest = &after[end + 1..];
            }
            None => {
                output.push_str(&rest[start..]);
                rest = "";
            }
        }
    }
    output.push_str(rest);
    output
}

fn trimmed_dir(value: &str) -> PathBuf {
    let value = value.trim().trim_matches('"');
    let mut trimmed = value.to_string();
    while trimmed.len() > 3 && (trimmed.ends_with('\\') || trimmed.ends_with('/')) {
        trimmed.pop();
    }
    PathBuf::from(trimmed)
}

fn same_path(left: Option<&Path>, right: Option<&Path>) -> bool {
    match (left, right) {
        (Some(left), Some(right)) => {
            normalize_entry(&left.to_string_lossy()) == normalize_entry(&right.to_string_lossy())
        }
        (None, None) => true,
        _ => false,
    }
}

fn has_executables(dir: &Path) -> bool {
    std::fs::read_dir(dir)
        .map(|entries| {
            entries.flatten().any(|entry| {
                entry
                    .file_name()
                    .to_string_lossy()
                    .to_lowercase()
                    .ends_with(".exe")
            })
        })
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::{expand_percent, normalize_entry, normalize_install_date};

    #[test]
    fn normalizes_path_entries() {
        assert_eq!(
            normalize_entry(r#""C:\Tools\RuxCmd\bin\\""#),
            r"c:\tools\ruxcmd\bin"
        );
        assert_eq!(normalize_entry(r"C:\"), r"c:\");
    }

    #[test]
    fn keeps_unknown_percent_variables() {
        assert_eq!(expand_percent("%RUXCMD_TEST_UNSET%"), "%RUXCMD_TEST_UNSET%");
    }

    #[test]
    fn accepts_only_well_formed_install_dates() {
        assert_eq!(
            normalize_install_date("20261109"),
            Some("20261109".to_string())
        );
        assert_eq!(
            normalize_install_date(" 20261109 "),
            Some("20261109".to_string())
        );
        for bad in [
            "",
            "2026-11-09",
            "2026119",
            "202611091",
            "2026110x",
            "20261309",
            "20261132",
            "19991231",
        ] {
            assert_eq!(
                normalize_install_date(bad),
                None,
                "unexpectedly accepted {bad:?}"
            );
        }
    }
}
