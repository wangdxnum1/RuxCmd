use serde::{Deserialize, Serialize};

/// How the suite was installed. `NotInstalled` is a normal state, not an error.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum InstallMethod {
    Script,
    Inno,
    Portable,
    #[default]
    NotInstalled,
    Unknown,
}

/// Result of checking files against `share/SHA256SUMS`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ChecksumStatus {
    Verified,
    Mismatch,
    Missing,
    #[default]
    NotChecked,
}

/// Unified read-only installation snapshot (design §4.1).
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallInfo {
    pub method: InstallMethod,
    pub suite_version: Option<String>,
    pub install_dir: Option<String>,
    pub source_commit: Option<String>,
    pub dirty: Option<bool>,
    pub target: Option<String>,
    pub rustc_version: Option<String>,
    pub binary_count: Option<usize>,
    pub shim_count: Option<usize>,
    /// Where `binary_count` and `shim_count` came from: state file, manifest,
    /// or a directory scan.
    pub counts_source: Option<String>,
    pub checksum_status: ChecksumStatus,
    pub path_on_user_path: Option<bool>,
    /// SHA256 of the release ZIP, as recorded by the script installer's state
    /// file. Inno installs do not record the package hash, so this stays None
    /// there; the UI shows the published release digest instead.
    pub package_sha256: Option<String>,
    /// ISO-8601 UTC timestamp from the script state file, or `YYYYMMDD` from
    /// the Inno uninstall key. `installed_at_kind` tells the UI which it is.
    pub installed_at: Option<String>,
    pub installed_at_kind: InstalledAtKind,
    pub conflicts: Vec<String>,
    pub warnings: Vec<String>,
}

/// Precision of `InstallInfo::installed_at`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum InstalledAtKind {
    /// No installation date available.
    #[default]
    Unknown,
    /// Full ISO-8601 UTC timestamp (script installer state file).
    Timestamp,
    /// Day-only `YYYYMMDD` value (Inno `InstallDate` registry value).
    Date,
}

/// Information about the Manager itself. The suite version lives in `InstallInfo`.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ManagerInfo {
    pub version: String,
    pub target: String,
    pub build_time: String,
    pub tauri_version: String,
    pub signed: bool,
    pub read_only: bool,
}

/// GitHub REST release object as returned by the API (snake_case fields).
#[derive(Debug, Clone, Deserialize)]
pub struct GhRelease {
    pub tag_name: String,
    pub name: Option<String>,
    pub prerelease: bool,
    pub draft: bool,
    pub published_at: Option<String>,
    pub html_url: String,
    pub body: Option<String>,
    pub assets: Vec<GhAsset>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GhAsset {
    pub name: String,
    pub size: u64,
    pub digest: Option<String>,
    pub state: String,
    #[serde(default)]
    pub browser_download_url: Option<String>,
}

/// Release data normalized for the UI (camelCase fields).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReleaseInfo {
    pub tag: String,
    pub version: String,
    pub name: Option<String>,
    pub prerelease: bool,
    pub draft: bool,
    pub published_at: Option<String>,
    pub html_url: String,
    pub body: Option<String>,
    pub assets: Vec<ReleaseAsset>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReleaseAsset {
    pub name: String,
    pub size: u64,
    pub digest: Option<String>,
    pub state: String,
    /// Used by the backend to fetch the `.sha256` sidecar; never exposed to the UI.
    #[serde(skip_serializing)]
    pub download_url: Option<String>,
}

/// Result of comparing the API's asset digest with the hash published inside
/// the matching `.sha256` sidecar file.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SidecarCheck {
    pub binary: String,
    pub sidecar: String,
    /// SHA-256 reported by the GitHub API for the binary asset.
    pub digest_sha256: Option<String>,
    /// SHA-256 written inside the sidecar file.
    pub sidecar_sha256: Option<String>,
    /// `None` means the comparison could not be performed (see `error`).
    pub matches: Option<bool>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum UpdateStatus {
    UpToDate,
    UpdateAvailable,
}

/// Read-only result of `check_update`. Errors are transported separately.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateCheck {
    pub status: UpdateStatus,
    pub current_version: Option<String>,
    pub latest: Option<ReleaseInfo>,
    pub assets_complete: bool,
    pub missing_assets: Vec<String>,
    pub sidecar_checks: Vec<SidecarCheck>,
    pub checked_at: String,
    pub from_cache: bool,
    pub next_refresh_seconds: u64,
}

/// Typed failure for `check_update` and the P2 update commands. The UI must
/// never render an error as "up to date".
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateCheckError {
    /// `offline`, `rateLimited`, `notFound`, `invalidResponse`, `httpError`,
    /// `unsupported`, `needsConfirmation`, `cancelled`, `internal`.
    pub kind: String,
    pub message: String,
    pub retry_after_seconds: Option<u64>,
    /// Extra lines `apply_update` asks the UI to confirm before overwriting.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub details: Vec<String>,
    /// True when `apply_update` stopped because local files were modified.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub needs_confirmation: bool,
    /// True when a download or prepare was cancelled on purpose.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub cancelled: bool,
}

impl UpdateCheckError {
    fn new(kind: &str, message: impl Into<String>) -> Self {
        Self {
            kind: kind.to_string(),
            message: message.into(),
            retry_after_seconds: None,
            details: Vec::new(),
            needs_confirmation: false,
            cancelled: false,
        }
    }

    pub fn offline(message: impl Into<String>) -> Self {
        Self::new("offline", message)
    }

    pub fn rate_limited(message: impl Into<String>, retry_after_seconds: Option<u64>) -> Self {
        let mut error = Self::new("rateLimited", message);
        error.retry_after_seconds = retry_after_seconds;
        error
    }

    pub fn not_found(message: impl Into<String>) -> Self {
        Self::new("notFound", message)
    }

    pub fn invalid_response(message: impl Into<String>) -> Self {
        Self::new("invalidResponse", message)
    }

    pub fn http_error(message: impl Into<String>) -> Self {
        Self::new("httpError", message)
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Self::new("internal", message)
    }

    /// The installation method has no in-app upgrade path (portable, missing,
    /// or unknown).
    pub fn unsupported(message: impl Into<String>) -> Self {
        Self::new("unsupported", message)
    }

    /// `apply_update` stopped before touching the install directory.
    pub fn needs_confirmation(message: impl Into<String>, details: Vec<String>) -> Self {
        let mut error = Self::new("needsConfirmation", message);
        error.details = details;
        error.needs_confirmation = true;
        error
    }

    pub fn cancelled(message: impl Into<String>) -> Self {
        let mut error = Self::new("cancelled", message);
        error.cancelled = true;
        error
    }
}

impl std::fmt::Display for UpdateCheckError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}", self.message)
    }
}

/// Per-file integrity report returned by `verify_installation`.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VerificationReport {
    pub status: ChecksumStatus,
    pub checked: usize,
    pub missing: Vec<String>,
    pub mismatched: Vec<String>,
    pub source: Option<String>,
    pub message: Option<String>,
}

impl VerificationReport {
    pub fn not_checked(message: impl Into<String>) -> Self {
        Self {
            status: ChecksumStatus::NotChecked,
            checked: 0,
            missing: Vec::new(),
            mismatched: Vec::new(),
            source: None,
            message: Some(message.into()),
        }
    }
}

/// Which artifact the desktop client downloads for a self-update.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum UpdateKind {
    /// Script install: download the ZIP and let `install.ps1` replace the files.
    Script,
    /// Inno install: download `setup.exe` and run it silently.
    Inno,
}

/// Result of `prepare_update`. The artifact is downloaded and verified; the
/// install directory has not been touched.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PreparedUpdate {
    pub id: String,
    pub version: String,
    pub kind: UpdateKind,
    pub artifact: String,
    pub sha256: String,
    pub size: u64,
}

/// Result of `apply_update`. The helper script runs after the UI exits.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplyOutcome {
    pub handed_off: bool,
    pub kind: UpdateKind,
    pub version: String,
    /// Absolute path of the generated helper, for diagnostics only.
    pub helper: Option<String>,
}

/// `update-progress` payload.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateProgress {
    pub id: String,
    pub stage: String,
    pub received: u64,
    pub total: u64,
}

/// `update-stage` payload.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateStage {
    pub id: String,
    pub stage: String,
}

/// `update-log` payload.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateLog {
    pub id: String,
    pub line: String,
}

/// `update-finished` payload, emitted once per `apply_update` call.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateFinished {
    pub id: String,
    /// `handedOff` or `failed`.
    pub outcome: String,
    pub version: Option<String>,
    /// Backup directory reported to the user (script installs only).
    pub rollback: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::ReleaseInfo;

    /// The Tauri command layer serializes with `rename_all = "camelCase"`.
    /// `desktop/ui/app.js` reads `latest.htmlUrl` when wiring the release-page
    /// button; a drift back to `html_url` would silently set its `data-url`
    /// attribute to the string "undefined" and break the button (the browser
    /// preview never calls open_external, so no JS test would catch it).
    /// This pins both halves of that contract from the Rust side.
    #[test]
    fn release_info_serializes_html_url_as_camel_case() {
        let info = ReleaseInfo {
            tag: "v0.5.12".to_string(),
            version: "0.5.12".to_string(),
            name: None,
            prerelease: false,
            draft: false,
            published_at: None,
            html_url: "https://github.com/wangdxnum1/RuxCmd/releases/tag/v0.5.12".to_string(),
            body: None,
            assets: Vec::new(),
        };
        let json = serde_json::to_value(&info).expect("serialize ReleaseInfo");
        assert!(
            json.get("htmlUrl").is_some(),
            "ReleaseInfo must expose htmlUrl for app.js: {json}"
        );
        assert!(
            json.get("html_url").is_none(),
            "ReleaseInfo must not expose snake_case html_url: {json}"
        );
        assert_eq!(
            json["htmlUrl"],
            "https://github.com/wangdxnum1/RuxCmd/releases/tag/v0.5.12"
        );
    }
}
