use std::time::{Duration, Instant};

use semver::Version;

use crate::model::{
    GhRelease, ReleaseAsset, ReleaseInfo, SidecarCheck, UpdateCheck, UpdateCheckError, UpdateStatus,
};
use crate::util;

const API_BASE: &str = "https://api.github.com";
const API_LATEST_PATH: &str = "/repos/wangdxnum1/RuxCmd/releases/latest";
const API_LIST_PATH: &str = "/repos/wangdxnum1/RuxCmd/releases?per_page=20";
const TARGET: &str = "x86_64-pc-windows-msvc";

/// Domains the desktop client is allowed to contact (design §6). GitHub now
/// redirects release assets to `release-assets.githubusercontent.com`.
const ALLOWED_HOSTS: [&str; 4] = [
    "api.github.com",
    "github.com",
    "objects.githubusercontent.com",
    "release-assets.githubusercontent.com",
];

/// Minimum interval between two requests to the same GitHub endpoint. Keeps the
/// Manager well inside the unauthenticated 60 requests/hour budget.
pub const MIN_REFRESH_SECONDS: u64 = 600;

#[derive(Debug, Clone)]
pub struct CacheEntry {
    pub release: ReleaseInfo,
    pub etag: Option<String>,
    pub sidecar_checks: Vec<SidecarCheck>,
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

/// Read-only release probe. Performs the API request plus the two `.sha256`
/// sidecar requests on a fresh fetch, and reuses the whole cached result when a
/// 304 is returned.
pub fn check(
    cache: &mut ReleaseCache,
    include_prerelease: bool,
    current_version: Option<&str>,
) -> Result<UpdateCheck, UpdateCheckError> {
    check_at(API_BASE, false, cache, include_prerelease, current_version)
}

fn check_at(
    base: &str,
    allow_insecure_http: bool,
    cache: &mut ReleaseCache,
    include_prerelease: bool,
    current_version: Option<&str>,
) -> Result<UpdateCheck, UpdateCheckError> {
    let path = if include_prerelease {
        API_LIST_PATH
    } else {
        API_LATEST_PATH
    };
    let url = format!("{}{}", base.trim_end_matches('/'), path);
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
    let outcome = fetch(&url, etag.as_deref(), allow_insecure_http)?;
    let (body, etag) = match outcome {
        Fetch::NotModified => match slot {
            Some(entry) => {
                entry.fetched_at = Instant::now();
                return Ok(build_check(
                    entry,
                    current_version,
                    true,
                    MIN_REFRESH_SECONDS,
                ));
            }
            None => match fetch(&url, None, allow_insecure_http)? {
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
    let sidecar_checks = build_sidecar_checks(&release, allow_insecure_http);
    let entry = CacheEntry {
        release,
        etag,
        sidecar_checks,
        checked_at: util::utc_now_iso(),
        fetched_at: Instant::now(),
    };
    let check = build_check(&entry, current_version, false, MIN_REFRESH_SECONDS);
    *slot = Some(entry);
    Ok(check)
}

fn fetch(
    url: &str,
    etag: Option<&str>,
    allow_insecure_http: bool,
) -> Result<Fetch, UpdateCheckError> {
    fetch_hops(url, etag, allow_insecure_http, 0)
}

fn fetch_hops(
    url: &str,
    etag: Option<&str>,
    allow_insecure_http: bool,
    hops: u8,
) -> Result<Fetch, UpdateCheckError> {
    ensure_allowed_url(url, allow_insecure_http)?;

    let mut builder = ureq::AgentBuilder::new()
        .timeout(Duration::from_secs(20))
        .redirects(0);
    if let Some(proxy) = system_proxy(allow_insecure_http) {
        builder = builder.proxy(proxy);
    }
    let agent = builder.build();
    let mut request = agent
        .get(url)
        .set(
            "User-Agent",
            concat!("RuxCmd-Desktop/", env!("CARGO_PKG_VERSION")),
        )
        .set("Accept", "application/vnd.github+json")
        .set("X-GitHub-Api-Version", "2022-11-28");
    if let Some(etag) = etag {
        request = request.set("If-None-Match", etag);
    }

    let response = match request.call() {
        Ok(response) => response,
        Err(ureq::Error::Status(code, response)) => return Err(classify_status(code, &response)),
        Err(ureq::Error::Transport(transport)) => {
            let host = url::Url::parse(url)
                .ok()
                .and_then(|parsed| parsed.host_str().map(str::to_string))
                .unwrap_or_else(|| "GitHub API".to_string());
            return Err(UpdateCheckError::offline(format!(
                "无法连接 {host}：{transport}"
            )));
        }
    };

    let status = response.status();
    if status == 304 {
        return Ok(Fetch::NotModified);
    }
    if (300..400).contains(&status)
        && let Some(location) = response.header("location")
    {
        if hops >= 4 {
            return Err(UpdateCheckError::invalid_response(format!(
                "重定向次数过多，已停止：{url}"
            )));
        }
        let next = url::Url::parse(url)
            .and_then(|base| base.join(location))
            .map_err(|error| {
                UpdateCheckError::invalid_response(format!("重定向地址无法解析：{error}"))
            })?;
        return fetch_hops(next.as_str(), None, allow_insecure_http, hops + 1);
    }
    let etag = response.header("etag").map(str::to_string);
    let body = response
        .into_string()
        .map_err(|error| UpdateCheckError::invalid_response(format!("读取响应失败：{error}")))?;
    Ok(Fetch::Body { body, etag })
}

/// Honors an explicit proxy (env vars first, then the Windows system proxy)
/// so `github.com` asset downloads work on machines that only reach GitHub
/// through a local proxy. Tests opt out via `allow_insecure_http`.
fn system_proxy(allow_insecure_http: bool) -> Option<ureq::Proxy> {
    if allow_insecure_http {
        return None;
    }
    let raw = std::env::var("HTTPS_PROXY")
        .ok()
        .or_else(|| std::env::var("https_proxy").ok())
        .or_else(|| std::env::var("HTTP_PROXY").ok())
        .or_else(|| std::env::var("http_proxy").ok())
        .or_else(windows_system_proxy)?;
    let url = normalize_proxy_url(&raw)?;
    ureq::Proxy::new(&url).ok()
}

fn normalize_proxy_url(raw: &str) -> Option<String> {
    let raw = raw.trim();
    if raw.is_empty() {
        return None;
    }
    if raw.contains("://") {
        return Some(raw.to_string());
    }
    for scheme in ["https=", "http="] {
        if let Some(value) = raw
            .split(';')
            .find_map(|part| part.trim().strip_prefix(scheme))
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            return Some(format!("http://{value}"));
        }
    }
    Some(format!("http://{raw}"))
}

#[cfg(windows)]
fn windows_system_proxy() -> Option<String> {
    use winreg::RegKey;
    use winreg::enums::HKEY_CURRENT_USER;

    let key = RegKey::predef(HKEY_CURRENT_USER)
        .open_subkey("Software\\Microsoft\\Windows\\CurrentVersion\\Internet Settings")
        .ok()?;
    let enabled: u32 = key.get_value("ProxyEnable").ok()?;
    if enabled == 0 {
        return None;
    }
    key.get_value("ProxyServer").ok()
}

#[cfg(not(windows))]
fn windows_system_proxy() -> Option<String> {
    None
}

fn ensure_allowed_url(url: &str, allow_insecure_http: bool) -> Result<(), UpdateCheckError> {
    let parsed = url::Url::parse(url)
        .map_err(|error| UpdateCheckError::invalid_response(format!("URL 无法解析：{error}")))?;
    let scheme = parsed.scheme();
    let host = parsed.host_str().unwrap_or_default();
    let scheme_ok = scheme == "https" || (allow_insecure_http && scheme == "http");
    let host_ok = ALLOWED_HOSTS.contains(&host) || (allow_insecure_http && scheme == "http");
    if scheme_ok && host_ok {
        return Ok(());
    }
    Err(UpdateCheckError::invalid_response(format!(
        "拒绝访问不在允许列表内的地址：{scheme}://{host}"
    )))
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
                "GitHub API 限流（未认证 60 次/小时），此时无法判断版本新旧",
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

    let release: GhRelease = serde_json::from_str(body).map_err(|error| {
        UpdateCheckError::invalid_response(format!("发布信息无法解析：{error}"))
    })?;
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
                download_url: asset.browser_download_url,
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

/// Downloads both `.sha256` sidecars and compares their embedded hash with the
/// digest GitHub reports for the matching binary. Any problem becomes a per-asset
/// `error` instead of failing the whole version check.
fn build_sidecar_checks(release: &ReleaseInfo, allow_insecure_http: bool) -> Vec<SidecarCheck> {
    let names = expected_assets(&release.version);
    let mut checks = Vec::new();
    for binary in [names[0].clone(), names[2].clone()] {
        let sidecar_name = format!("{binary}.sha256");
        let binary_asset = release
            .assets
            .iter()
            .find(|asset| asset.name == binary && asset.state == "uploaded");
        let sidecar_asset = release
            .assets
            .iter()
            .find(|asset| asset.name == sidecar_name && asset.state == "uploaded");
        let digest_sha256 = binary_asset
            .and_then(|asset| asset.digest.as_deref())
            .map(normalize_digest);

        let mut check = SidecarCheck {
            binary: binary.clone(),
            sidecar: sidecar_name,
            digest_sha256: digest_sha256.clone(),
            sidecar_sha256: None,
            matches: None,
            error: None,
        };

        if binary_asset.is_none() {
            check.error = Some(format!("缺少或未上传的资产：{binary}"));
            checks.push(check);
            continue;
        }
        let Some(digest) = digest_sha256 else {
            check.error = Some(format!("GitHub API 未提供 digest：{binary}"));
            checks.push(check);
            continue;
        };
        let Some(sidecar_asset) = sidecar_asset else {
            check.error = Some(format!("缺少或未上传的资产：{}", check.sidecar));
            checks.push(check);
            continue;
        };
        let Some(url) = sidecar_asset.download_url.as_deref() else {
            check.error = Some(format!("资产缺少下载地址：{}", check.sidecar));
            checks.push(check);
            continue;
        };

        match fetch(url, None, allow_insecure_http) {
            Ok(Fetch::Body { body, .. }) => match parse_sidecar_hash(&body) {
                Ok(hash) => {
                    check.matches = Some(digest == hash);
                    check.sidecar_sha256 = Some(hash);
                }
                Err(error) => check.error = Some(error),
            },
            Ok(Fetch::NotModified) => {
                check.error = Some("sidecar 返回 304，但未发送条件请求".to_string());
            }
            Err(error) => check.error = Some(format!("下载 sidecar 失败：{error}")),
        }
        checks.push(check);
    }
    checks
}

fn normalize_digest(value: &str) -> String {
    value
        .trim()
        .strip_prefix("sha256:")
        .unwrap_or(value.trim())
        .to_ascii_lowercase()
}

fn parse_sidecar_hash(body: &str) -> Result<String, String> {
    let token = body
        .split_whitespace()
        .next()
        .ok_or_else(|| "sidecar 内容为空".to_string())?
        .to_ascii_lowercase();
    if token.len() != 64 || !token.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(format!("sidecar 内容不是 SHA-256：{token}"));
    }
    Ok(token)
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
        sidecar_checks: entry.sidecar_checks.clone(),
        checked_at: entry.checked_at.clone(),
        from_cache,
        next_refresh_seconds,
    }
}

#[cfg(test)]
mod tests {
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::sync::{Arc, Mutex};
    use std::thread::JoinHandle;

    use serde_json::json;

    use super::{
        MIN_REFRESH_SECONDS, ReleaseCache, check_at, ensure_allowed_url, normalize_proxy_url,
        parse_sidecar_hash,
    };
    use crate::model::UpdateStatus;

    const VERSION: &str = "0.5.11";
    const ZIP: &str = "ruxcmd-0.5.11-x86_64-pc-windows-msvc.zip";
    const ZIP_SHA: &str = "ruxcmd-0.5.11-x86_64-pc-windows-msvc.zip.sha256";
    const SETUP: &str = "ruxcmd-0.5.11-x86_64-pc-windows-msvc-setup.exe";
    const SETUP_SHA: &str = "ruxcmd-0.5.11-x86_64-pc-windows-msvc-setup.exe.sha256";

    fn hash(byte: char) -> String {
        std::iter::repeat_n(byte, 64).collect()
    }

    fn asset_json(base: &str, name: &str, digest: Option<&str>, state: &str) -> serde_json::Value {
        json!({
            "name": name,
            "size": 12,
            "digest": digest.map(|value| format!("sha256:{value}")),
            "state": state,
            "browser_download_url": format!("{base}/assets/{name}"),
        })
    }

    fn release_json(assets: &[serde_json::Value]) -> String {
        json!({
            "tag_name": format!("v{VERSION}"),
            "name": format!("RuxCmd v{VERSION}"),
            "prerelease": false,
            "draft": false,
            "published_at": "2026-11-07T15:21:54Z",
            "html_url": "https://github.com/wangdxnum1/RuxCmd/releases/tag/v0.5.11",
            "body": "notes",
            "assets": assets,
        })
        .to_string()
    }

    struct MockResponse {
        status: u16,
        headers: Vec<(String, String)>,
        body: String,
    }

    impl MockResponse {
        fn new(status: u16, body: impl Into<String>) -> Self {
            Self {
                status,
                headers: Vec::new(),
                body: body.into(),
            }
        }

        fn ok(body: impl Into<String>) -> Self {
            Self::new(200, body)
        }

        fn header(mut self, name: &str, value: &str) -> Self {
            self.headers.push((name.to_string(), value.to_string()));
            self
        }
    }

    #[derive(Clone)]
    struct RequestLog {
        path: String,
        headers: String,
    }

    struct MockServer {
        base: String,
        requests: Arc<Mutex<Vec<RequestLog>>>,
        _thread: JoinHandle<()>,
    }

    impl MockServer {
        fn start<F>(handler: F) -> Self
        where
            F: Fn(&str, &str, &str) -> MockResponse + Send + 'static,
        {
            let listener = TcpListener::bind("127.0.0.1:0").expect("bind mock server");
            let base = format!("http://{}", listener.local_addr().expect("mock address"));
            let requests = Arc::new(Mutex::new(Vec::new()));
            let log = Arc::clone(&requests);
            let base_for_thread = base.clone();
            let thread = std::thread::spawn(move || {
                for stream in listener.incoming() {
                    let Ok(mut stream) = stream else {
                        continue;
                    };
                    let mut buffer = [0u8; 4096];
                    let read = stream.read(&mut buffer).unwrap_or(0);
                    let text = String::from_utf8_lossy(&buffer[..read]).to_string();
                    let mut lines = text.lines();
                    let request_line = lines.next().unwrap_or_default();
                    let path = request_line
                        .split_whitespace()
                        .nth(1)
                        .unwrap_or_default()
                        .to_string();
                    let headers = lines
                        .take_while(|line| !line.is_empty())
                        .collect::<Vec<_>>()
                        .join("\n");
                    log.lock().unwrap().push(RequestLog {
                        path: path.clone(),
                        headers: headers.clone(),
                    });
                    let response = handler(&base_for_thread, &path, &headers);
                    let mut raw = format!(
                        "HTTP/1.1 {} {}\r\nContent-Length: {}\r\nConnection: close\r\n",
                        response.status,
                        status_text(response.status),
                        response.body.len()
                    );
                    for (name, value) in response.headers {
                        raw.push_str(&format!("{name}: {value}\r\n"));
                    }
                    raw.push_str("\r\n");
                    raw.push_str(&response.body);
                    let _ = stream.write_all(raw.as_bytes());
                }
            });
            Self {
                base,
                requests,
                _thread: thread,
            }
        }

        fn request_paths(&self) -> Vec<String> {
            self.requests
                .lock()
                .unwrap()
                .iter()
                .map(|entry| entry.path.clone())
                .collect()
        }
    }

    fn status_text(code: u16) -> &'static str {
        match code {
            200 => "OK",
            304 => "Not Modified",
            404 => "Not Found",
            500 => "Internal Server Error",
            _ => "Unknown",
        }
    }

    #[test]
    fn happy_path_matches_digests_with_sidecars() {
        let server = MockServer::start(move |base, path, _headers| {
            if path == "/repos/wangdxnum1/RuxCmd/releases/latest" {
                MockResponse::ok(release_json(&[
                    asset_json(base, ZIP, Some(&hash('a')), "uploaded"),
                    asset_json(base, ZIP_SHA, Some(&hash('c')), "uploaded"),
                    asset_json(base, SETUP, Some(&hash('a')), "uploaded"),
                    asset_json(base, SETUP_SHA, Some(&hash('d')), "uploaded"),
                ]))
                .header("ETag", "\"mock-etag\"")
            } else if path.ends_with(".zip.sha256") {
                MockResponse::ok(format!("{}  {ZIP}\r\n", hash('a')))
            } else if path.ends_with("-setup.exe.sha256") {
                MockResponse::ok(format!("{}  {SETUP}\r\n", hash('a')))
            } else {
                MockResponse::new(404, "not found")
            }
        });

        let mut cache = ReleaseCache::default();
        let result = check_at(&server.base, true, &mut cache, false, Some("0.5.10"))
            .expect("happy path should succeed");
        assert_eq!(result.status, UpdateStatus::UpdateAvailable);
        let latest = result.latest.as_ref().expect("latest");
        assert_eq!(latest.version, VERSION);
        assert_eq!(latest.assets.len(), 4);
        assert!(result.assets_complete);
        assert_eq!(result.sidecar_checks.len(), 2);
        for check in &result.sidecar_checks {
            assert_eq!(check.matches, Some(true), "{:?}", check.error);
        }
        let paths = server.request_paths();
        assert_eq!(paths.len(), 3, "API + two sidecars: {paths:?}");
        assert!(paths.iter().any(|path| path.ends_with(".zip.sha256")));
    }

    #[test]
    fn etag_304_reuses_sidecar_results_without_refetching() {
        let server = MockServer::start(|base, path, headers| {
            if path == "/repos/wangdxnum1/RuxCmd/releases/latest" {
                if headers
                    .to_lowercase()
                    .contains("if-none-match: \"mock-etag\"")
                {
                    return MockResponse::new(304, "");
                }
                MockResponse::ok(release_json(&[
                    asset_json(base, ZIP, Some(&hash('a')), "uploaded"),
                    asset_json(base, ZIP_SHA, Some(&hash('c')), "uploaded"),
                    asset_json(base, SETUP, Some(&hash('a')), "uploaded"),
                    asset_json(base, SETUP_SHA, Some(&hash('d')), "uploaded"),
                ]))
                .header("ETag", "\"mock-etag\"")
            } else if path.ends_with(".zip.sha256") {
                MockResponse::ok(format!("{}  {ZIP}\r\n", hash('a')))
            } else if path.ends_with("-setup.exe.sha256") {
                MockResponse::ok(format!("{}  {SETUP}\r\n", hash('a')))
            } else {
                MockResponse::new(404, "not found")
            }
        });

        let mut cache = ReleaseCache::default();
        let first =
            check_at(&server.base, true, &mut cache, false, Some("0.5.11")).expect("first check");
        assert!(!first.from_cache);
        assert_eq!(server.request_paths().len(), 3);

        if let Some(entry) = cache.stable.as_mut() {
            entry.fetched_at = std::time::Instant::now()
                - std::time::Duration::from_secs(MIN_REFRESH_SECONDS + 30);
        }
        let second =
            check_at(&server.base, true, &mut cache, false, Some("0.5.11")).expect("second check");
        assert!(second.from_cache, "a 304 must be reused from cache");
        assert!(
            second
                .sidecar_checks
                .iter()
                .all(|check| check.matches == Some(true))
        );
        let requests = server.requests.lock().unwrap().clone();
        assert_eq!(requests.len(), 4, "only the API request is repeated");
        let last = requests.last().unwrap();
        assert_eq!(last.path, "/repos/wangdxnum1/RuxCmd/releases/latest");
        assert!(
            last.headers
                .to_lowercase()
                .contains("if-none-match: \"mock-etag\""),
            "second request headers: {}",
            last.headers
        );
    }

    #[test]
    fn missing_asset_is_incomplete_even_when_version_matches() {
        let server = MockServer::start(|base, path, _headers| {
            if path == "/repos/wangdxnum1/RuxCmd/releases/latest" {
                let assets = vec![
                    asset_json(base, ZIP, Some(&hash('a')), "uploaded"),
                    asset_json(base, ZIP_SHA, Some(&hash('c')), "uploaded"),
                    asset_json(base, SETUP_SHA, Some(&hash('d')), "uploaded"),
                ];
                MockResponse::ok(release_json(&assets))
            } else if path.ends_with(".zip.sha256") {
                MockResponse::ok(format!("{}  {ZIP}\r\n", hash('a')))
            } else {
                MockResponse::new(404, "not found")
            }
        });

        let mut cache = ReleaseCache::default();
        let result = check_at(&server.base, true, &mut cache, false, Some("0.5.11"))
            .expect("check should succeed with an integrity problem");
        assert_eq!(result.status, UpdateStatus::UpToDate);
        assert!(!result.assets_complete);
        assert!(result.missing_assets.contains(&SETUP.to_string()));
        let setup_check = result
            .sidecar_checks
            .iter()
            .find(|check| check.binary == SETUP)
            .expect("setup check");
        assert_eq!(setup_check.matches, None);
        assert!(
            setup_check
                .error
                .as_deref()
                .unwrap_or_default()
                .contains("缺少或未上传")
        );
    }

    #[test]
    fn sidecar_hash_mismatch_is_reported() {
        let server = MockServer::start(|base, path, _headers| {
            if path == "/repos/wangdxnum1/RuxCmd/releases/latest" {
                MockResponse::ok(release_json(&[
                    asset_json(base, ZIP, Some(&hash('a')), "uploaded"),
                    asset_json(base, ZIP_SHA, Some(&hash('c')), "uploaded"),
                    asset_json(base, SETUP, Some(&hash('a')), "uploaded"),
                    asset_json(base, SETUP_SHA, Some(&hash('d')), "uploaded"),
                ]))
            } else if path.ends_with(".zip.sha256") {
                MockResponse::ok(format!("{}  {ZIP}\r\n", hash('b')))
            } else if path.ends_with("-setup.exe.sha256") {
                MockResponse::ok(format!("{}  {SETUP}\r\n", hash('a')))
            } else {
                MockResponse::new(404, "not found")
            }
        });

        let mut cache = ReleaseCache::default();
        let result = check_at(&server.base, true, &mut cache, false, Some("0.5.11"))
            .expect("check should succeed with a mismatch");
        let zip_check = result
            .sidecar_checks
            .iter()
            .find(|check| check.binary == ZIP)
            .expect("zip check");
        assert_eq!(zip_check.matches, Some(false));
        assert_eq!(
            zip_check.sidecar_sha256.as_deref(),
            Some(hash('b').as_str())
        );
    }

    #[test]
    fn http_404_is_not_found() {
        let server =
            MockServer::start(|_base, _path, _headers| MockResponse::new(404, "not found"));
        let mut cache = ReleaseCache::default();
        let error = check_at(&server.base, true, &mut cache, false, Some("0.5.11"))
            .expect_err("404 must fail");
        assert_eq!(error.kind, "notFound");
        assert!(error.message.contains("404"));
    }

    #[test]
    fn offline_is_reported_as_offline() {
        let mut cache = ReleaseCache::default();
        let error = check_at(
            "http://127.0.0.1:1",
            true,
            &mut cache,
            false,
            Some("0.5.11"),
        )
        .expect_err("connection refused");
        assert_eq!(error.kind, "offline");
    }

    #[test]
    fn non_allowlisted_host_is_refused() {
        let mut cache = ReleaseCache::default();
        let error = check_at(
            "https://example.com",
            false,
            &mut cache,
            false,
            Some("0.5.11"),
        )
        .expect_err("host not allowed");
        assert_eq!(error.kind, "invalidResponse");
        assert!(error.message.contains("允许列表"));
    }

    #[test]
    fn sidecar_body_must_start_with_a_sha256() {
        assert!(parse_sidecar_hash("not-a-hash file.zip").is_err());
        assert_eq!(
            parse_sidecar_hash(&format!("{}  file.zip\r\n", hash('a'))).unwrap(),
            hash('a')
        );
    }

    #[test]
    fn release_asset_hosts_are_allowlisted() {
        assert!(
            ensure_allowed_url("https://release-assets.githubusercontent.com/x", false).is_ok()
        );
        assert!(ensure_allowed_url("https://objects.githubusercontent.com/x", false).is_ok());
        assert!(ensure_allowed_url("https://example.com/x", false).is_err());
    }

    #[test]
    fn follows_redirects_within_the_allowlist() {
        let server = MockServer::start(|base, path, _headers| {
            if path == "/repos/wangdxnum1/RuxCmd/releases/latest" {
                MockResponse::ok(release_json(&[
                    asset_json(base, ZIP, Some(&hash('a')), "uploaded"),
                    asset_json(base, ZIP_SHA, Some(&hash('c')), "uploaded"),
                    asset_json(base, SETUP, Some(&hash('a')), "uploaded"),
                    asset_json(base, SETUP_SHA, Some(&hash('d')), "uploaded"),
                ]))
            } else if path == "/real/zip.sha256" {
                MockResponse::ok(format!("{}  {ZIP}\r\n", hash('a')))
            } else if path.ends_with(".zip.sha256") {
                MockResponse::new(302, "").header("Location", &format!("{base}/real/zip.sha256"))
            } else if path.ends_with("-setup.exe.sha256") {
                MockResponse::ok(format!("{}  {SETUP}\r\n", hash('a')))
            } else {
                MockResponse::new(404, "not found")
            }
        });

        let mut cache = ReleaseCache::default();
        let result = check_at(&server.base, true, &mut cache, false, Some("0.5.11"))
            .expect("redirect should be followed");
        assert!(
            result
                .sidecar_checks
                .iter()
                .all(|check| check.matches == Some(true))
        );
        assert!(
            server
                .request_paths()
                .iter()
                .any(|path| path == "/real/zip.sha256")
        );
    }

    #[test]
    fn normalizes_proxy_forms() {
        assert_eq!(
            normalize_proxy_url("127.0.0.1:7890").as_deref(),
            Some("http://127.0.0.1:7890")
        );
        assert_eq!(
            normalize_proxy_url("http=127.0.0.1:7890;https=127.0.0.1:7890").as_deref(),
            Some("http://127.0.0.1:7890")
        );
        assert_eq!(
            normalize_proxy_url("http://proxy.example:8080").as_deref(),
            Some("http://proxy.example:8080")
        );
        assert!(normalize_proxy_url("   ").is_none());
    }

    /// Real GitHub call used for the P1 end-to-end verification. Run with:
    /// `cargo test real_github -- --ignored --nocapture`
    #[test]
    #[ignore = "performs real network requests to api.github.com"]
    fn real_github_check() {
        let mut cache = ReleaseCache::default();
        let first = check_at(
            "https://api.github.com",
            false,
            &mut cache,
            false,
            Some("0.5.10"),
        )
        .expect("live GitHub check");
        println!("{}", serde_json::to_string_pretty(&first).unwrap());
        let latest = first.latest.as_ref().expect("latest release");
        println!("latest tag={} version={}", latest.tag, latest.version);
        for asset in &latest.assets {
            println!(
                "asset {} state={} digest={:?} url={:?}",
                asset.name, asset.state, asset.digest, asset.download_url
            );
        }
        for check in &first.sidecar_checks {
            println!(
                "sidecar {} <-> {} matches={:?} api={:?} sidecar={:?} error={:?}",
                check.binary,
                check.sidecar,
                check.matches,
                check.digest_sha256,
                check.sidecar_sha256,
                check.error
            );
        }
        assert_eq!(first.status, UpdateStatus::UpdateAvailable);
        assert!(first.assets_complete, "missing: {:?}", first.missing_assets);
        for check in &first.sidecar_checks {
            assert_eq!(
                check.matches,
                Some(true),
                "{} sidecar mismatch: {:?}",
                check.binary,
                check.error
            );
        }

        if let Some(entry) = cache.stable.as_mut() {
            entry.fetched_at = std::time::Instant::now()
                - std::time::Duration::from_secs(MIN_REFRESH_SECONDS + 30);
        }
        let second = check_at(
            "https://api.github.com",
            false,
            &mut cache,
            false,
            Some("0.5.10"),
        )
        .expect("live GitHub conditional check");
        println!(
            "second from_cache={} (true means GitHub returned 304 to If-None-Match)",
            second.from_cache
        );
        assert!(
            second.from_cache,
            "expected a real 304 for the second request"
        );
    }
}
