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
    pub package_sha256: Option<String>,
    pub installed_at: Option<String>,
    pub conflicts: Vec<String>,
    pub warnings: Vec<String>,
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

/// Typed failure for `check_update`. The UI must never render an error as
/// "up to date".
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateCheckError {
    /// `offline`, `rateLimited`, `notFound`, `invalidResponse`, `httpError`.
    pub kind: String,
    pub message: String,
    pub retry_after_seconds: Option<u64>,
}

impl UpdateCheckError {
    fn new(kind: &str, message: impl Into<String>) -> Self {
        Self {
            kind: kind.to_string(),
            message: message.into(),
            retry_after_seconds: None,
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
