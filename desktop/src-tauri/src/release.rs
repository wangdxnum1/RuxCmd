use std::time::{Duration, Instant};

use semver::Version;

use crate::model::{
    GhRelease, ReleaseAsset, ReleaseInfo, UpdateCheck, UpdateCheckError, UpdateStatus,
};
use crate::util;

const API_LATEST: &str = "https://api.github.com/repos/wangdxnum1/RuxCmd/releases/latest";
const API_LIST: &str = "https://api.github.com/repos/wangdxnum1/RuxCmd/releases?per_page=20";
const TARGET: &str = "x86_64-pc-windows-msvc";

/// Minimum interval between two requests to the same GitHub endpoint. Keeps the
/// Manager well inside the unauthenticated 60 requests/hour budget.
pub const MIN_REFRESH_SECONDS: u64 = 600;

#[derive(Debug, Clone)]
pub struct CacheEntry {
    pub release: ReleaseInfo,
    pub etag: Option<String>,
    pub checked_at: String,
    pub fetched_at: Instant,
}

/// In-memory only: P1 never writes files, so the ETag cache lasts one session.
#[derive(Debug, Default)]
pub struct ReleaseCache {
    pub stable: Option<CacheEntry>,
    pub any: Option<CacheEntry>,
}

enum Fetch {
    NotModified,
    Body { body: String, etag: Option<String> },
}

/// Read-only release probe. Performs at most one HTTP request per call and
/// reuses the cached response when a 304 is returned.
pub fn check(
    cache: &mut ReleaseCache,
    include_prerelease: bool,
    current_version: Option<&str>,
) -> Result<UpdateCheck, UpdateCheckError> {
    let url = if include_prerelease { API_LIST } else { API_LATEST };
    let slot = if include_prerelease {
        &mut cache.any
    } else {
        &mut cache.stable
    };

    if let Some(entry) = slot {
        let elapsed = entry.fetched_at.elapsed().as_secs();
        if elapsed < MIN_REFRESH_SECONDS {
            return Ok(build_check(
                entry,
                current_version,
                true,
                MIN_REFRESH_SECONDS - elapsed,
            ));
        }
    }

    let etag = slot.as_ref().and_then(|entry| entry.etag.clone());
    let outcome = fetch(url, etag.as_deref())?;
    let (body, etag) = match outcome {
        Fetch::NotModified => match slot {
            Some(entry) => {
                entry.fetched_at = Instant::now();
                return Ok(build_check(entry, current_version, true, MIN_REFRESH_SECONDS));
            }
            None => match fetch(url, None)? {
                Fetch::Body { body, etag } => (body, etag),
                Fetch::NotModified => {
                    return Err(UpdateCheckError::invalid_response(
                        "GitHub 返回 304，但本地没有缓存响应",
                    ));
                }
            },
        },
        Fetch::Body { body, etag } => (body, etag),
    };

    let release = parse_release(&body, include_prerelease)?;
    let entry = CacheEntry {
        release,
        etag,
        checked_at: util::utc_now_iso(),
        fetched_at: Instant::now(),
    };
    let check = build_check(&entry, current_version, false, MIN_REFRESH_SECONDS);
    *slot = Some(entry);
    Ok(check)
}

fn fetch(url: &str, etag: Option<&str>) -> Result<Fetch, UpdateCheckError> {
    let mut request = ureq::get(url)
        .set(
            "User-Agent",
            concat!("RuxCmd-Desktop/", env!("CARGO_PKG_VERSION")),
        )
        .set("Accept", "application/vnd.github+json")
        .set("X-GitHub-Api-Version", "2022-11-28")
        .timeout(Duration::from_secs(20));
    if let Some(etag) = etag {
        request = request.set("If-None-Match", etag);
    }

    let response = match request.call() {
        Ok(response) => response,
        Err(ureq::Error::Status(code, response)) => return Err(classify_status(code, &response)),
        Err(ureq::Error::Transport(transport)) => {
            return Err(UpdateCheckError::offline(format!(
                "无法连接 api.github.com：{transport}"
            )));
        }
    };

    if response.status() == 304 {
        return Ok(Fetch::NotModified);
    }
    let etag = response.header("etag").map(str::to_string);
    let body = response
        .into_string()
        .map_err(|error| UpdateCheckError::invalid_response(format!("读取响应失败：{error}")))?;
    Ok(Fetch::Body { body, etag })
}

fn classify_status(code: u16, response: &ureq::Response) -> UpdateCheckError {
    if code == 403 || code == 429 {
        let remaining = response
            .header("x-ratelimit-remaining")
            .and_then(|value| value.parse::<u64>().ok());
        if code == 429 || remaining == Some(0) {
            let reset = response
                .header("x-ratelimit-reset")
                .and_then(|value| value.parse::<u64>().ok());
            let retry_after = reset.map(|reset| reset.saturating_sub(util::unix_now_seconds()));
            return UpdateCheckError::rate_limited(
                "GitHub API 限流（未认证 60 次/小时），这不代表已是最新版本",
                retry_after,
            );
        }
    }
    if code == 404 {
        return UpdateCheckError::not_found("GitHub 上没有可用的发布记录（HTTP 404）");
    }
    UpdateCheckError::http_error(format!(
        "GitHub API 返回 HTTP {code}（{}）",
        response.status_text()
    ))
}

fn parse_release(body: &str, include_prerelease: bool) -> Result<ReleaseInfo, UpdateCheckError> {
    if include_prerelease {
        let releases: Vec<GhRelease> = serde_json::from_str(body).map_err(|error| {
            UpdateCheckError::invalid_response(format!("发布列表无法解析：{error}"))
        })?;
        let best = releases
            .into_iter()
            .filter(|release| !release.draft)
            .filter_map(|release| version_of(&release.tag_name).map(|version| (version, release)))
            .max_by(|left, right| left.0.cmp(&right.0))
            .map(|(_, release)| release)
            .ok_or_else(|| UpdateCheckError::not_found("GitHub 上暂无可用的发布版本"))?;
        return Ok(normalize_release(best));
    }

    let release: GhRelease = serde_json::from_str(body)
        .map_err(|error| UpdateCheckError::invalid_response(format!("发布信息无法解析：{error}")))?;
    if release.draft {
        return Err(UpdateCheckError::invalid_response(
            "GitHub 返回了草稿发布，拒绝使用",
        ));
    }
    if version_of(&release.tag_name).is_none() {
        return Err(UpdateCheckError::invalid_response(format!(
            "发布 tag 不是语义化版本：{}",
            release.tag_name
        )));
    }
    Ok(normalize_release(release))
}

fn normalize_release(release: GhRelease) -> ReleaseInfo {
    let version = version_of(&release.tag_name)
        .map(|version| version.to_string())
        .unwrap_or_else(|| release.tag_name.trim().to_string());
    ReleaseInfo {
        tag: release.tag_name,
        version,
        name: release.name,
        prerelease: release.prerelease,
        draft: release.draft,
        published_at: release.published_at,
        html_url: release.html_url,
        body: release.body,
        assets: release
            .assets
            .into_iter()
            .map(|asset| ReleaseAsset {
                name: asset.name,
                size: asset.size,
                digest: asset.digest,
                state: asset.state,
            })
            .collect(),
    }
}

fn version_of(value: &str) -> Option<Version> {
    Version::parse(value.trim().trim_start_matches(&['v', 'V'][..])).ok()
}

fn expected_assets(version: &str) -> Vec<String> {
    vec![
        format!("ruxcmd-{version}-{TARGET}.zip"),
        format!("ruxcmd-{version}-{TARGET}.zip.sha256"),
        format!("ruxcmd-{version}-{TARGET}-setup.exe"),
        format!("ruxcmd-{version}-{TARGET}-setup.exe.sha256"),
    ]
}

fn build_check(
    entry: &CacheEntry,
    current_version: Option<&str>,
    from_cache: bool,
    next_refresh_seconds: u64,
) -> UpdateCheck {
    let latest = Version::parse(&entry.release.version).ok();
    let status = match (current_version.and_then(version_of), &latest) {
        (Some(current), Some(latest)) if current >= *latest => UpdateStatus::UpToDate,
        _ => UpdateStatus::UpdateAvailable,
    };
    let missing_assets = expected_assets(&entry.release.version)
        .into_iter()
        .filter(|name| {
            !entry
                .release
                .assets
                .iter()
                .any(|asset| &asset.name == name && asset.state == "uploaded")
        })
        .collect::<Vec<_>>();
    UpdateCheck {
        status,
        current_version: current_version.map(str::to_string),
        latest: Some(entry.release.clone()),
        assets_complete: missing_assets.is_empty(),
        missing_assets,
        checked_at: entry.checked_at.clone(),
        from_cache,
        next_refresh_seconds,
    }
}

#[cfg(test)]
mod tests {
    use super::{expected_assets, version_of};

    #[test]
    fn strips_v_prefix_and_compares_semantically() {
        assert_eq!(version_of("v0.5.11").unwrap().to_string(), "0.5.11");
        assert!(version_of("v0.10.0").unwrap() > version_of("v0.9.9").unwrap());
        assert!(version_of("0.5.12-rc.1").unwrap() > version_of("0.5.11").unwrap());
        assert!(version_of("not-a-version").is_none());
    }

    #[test]
    fn expects_the_four_release_assets() {
        let assets = expected_assets("0.5.11");
        assert_eq!(assets.len(), 4);
        assert!(assets[0].ends_with(".zip"));
        assert!(assets[2].ends_with("-setup.exe"));
    }
}
