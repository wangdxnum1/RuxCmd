use std::io::Read;
use std::path::{Component, Path, PathBuf};

use sha2::{Digest, Sha256};

use crate::model::{ChecksumStatus, InstallInfo, InstallMethod, VerificationReport};

/// Verify every file listed in `share/SHA256SUMS`. Read-only and streaming:
/// no temporary files are created.
pub fn verify(install: &InstallInfo) -> VerificationReport {
    if install.method == InstallMethod::NotInstalled {
        return VerificationReport::not_checked("未安装 RuxCmd，跳过逐文件校验");
    }
    let Some(dir) = install.install_dir.as_deref().map(PathBuf::from) else {
        return VerificationReport::not_checked("未检测到安装目录，跳过逐文件校验");
    };

    let sums_path = dir.join("share").join("SHA256SUMS");
    let text = match std::fs::read_to_string(&sums_path) {
        Ok(text) => text,
        Err(_) => {
            return VerificationReport::not_checked(
                "安装目录缺少 share/SHA256SUMS，无法逐文件校验",
            );
        }
    };
    let entries = match parse_sums(&text) {
        Ok(entries) => entries,
        Err(message) => return VerificationReport::not_checked(message),
    };

    let mut checked = 0usize;
    let mut missing = Vec::new();
    let mut mismatched = Vec::new();

    for (expected, relative) in entries {
        let Some(path) = installed_path(&dir, &relative) else {
            missing.push(relative);
            continue;
        };
        if !path.is_file() {
            missing.push(relative);
            continue;
        }
        match sha256_file(&path) {
            Ok(actual) if actual == expected => checked += 1,
            Ok(_) => mismatched.push(relative),
            Err(_) => mismatched.push(relative),
        }
    }

    let status = if !mismatched.is_empty() {
        ChecksumStatus::Mismatch
    } else if !missing.is_empty() {
        ChecksumStatus::Missing
    } else if checked > 0 {
        ChecksumStatus::Verified
    } else {
        ChecksumStatus::NotChecked
    };

    VerificationReport {
        status,
        checked,
        missing,
        mismatched,
        source: Some("share/SHA256SUMS".to_string()),
        message: None,
    }
}

fn parse_sums(text: &str) -> Result<Vec<(String, String)>, String> {
    let mut entries = Vec::new();
    for (index, line) in text.lines().enumerate() {
        let line = line.trim_end();
        if line.trim().is_empty() {
            continue;
        }
        if line.len() < 66 {
            return Err(format!("SHA256SUMS 第 {} 行格式无效", index + 1));
        }
        let (hash, rest) = line.split_at(64);
        if !hash.chars().all(|character| character.is_ascii_hexdigit()) {
            return Err(format!("SHA256SUMS 第 {} 行格式无效", index + 1));
        }
        let relative = rest.trim_start().trim_start_matches('*').trim_end();
        if relative.is_empty() {
            return Err(format!("SHA256SUMS 第 {} 行缺少路径", index + 1));
        }
        if installed_path(Path::new("."), relative).is_none() {
            return Err(format!("SHA256SUMS 第 {} 行包含不安全路径", index + 1));
        }
        entries.push((hash.to_ascii_lowercase(), relative.to_string()));
    }
    Ok(entries)
}

/// Map a package-relative path to its installed location: `bin/x.exe` is
/// flattened into the install directory; everything else lives under `share/`.
fn installed_path(dir: &Path, relative: &str) -> Option<PathBuf> {
    let relative = relative.replace('\\', "/");
    let relative_path = Path::new(&relative);
    if relative_path.is_absolute() {
        return None;
    }
    for component in relative_path.components() {
        if !matches!(component, Component::Normal(_)) {
            return None;
        }
    }
    let mut path = dir.to_path_buf();
    match relative.strip_prefix("bin/") {
        Some(name) => path.push(name),
        None => {
            path.push("share");
            path.push(&relative);
        }
    }
    Some(path)
}

fn sha256_file(path: &Path) -> std::io::Result<String> {
    let mut file = std::fs::File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 64 * 1024];
    loop {
        let read = file.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::{installed_path, parse_sums};

    #[test]
    fn maps_package_paths_to_installed_layout() {
        let root = Path::new(r"C:\RuxCmd\bin");
        assert_eq!(
            installed_path(root, "bin/find.exe").unwrap(),
            root.join("find.exe")
        );
        assert_eq!(
            installed_path(root, "README.md").unwrap(),
            root.join("share").join("README.md")
        );
    }

    #[test]
    fn rejects_unsafe_paths() {
        let root = Path::new(r"C:\RuxCmd\bin");
        assert!(installed_path(root, "../escape.exe").is_none());
        assert!(installed_path(root, "C:/Windows/system32/cmd.exe").is_none());
        assert!(installed_path(root, "/abs/path").is_none());
    }

    #[test]
    fn parses_sha256sums_lines() {
        let text =
            "ABCDEF0123456789ABCDEF0123456789ABCDEF0123456789ABCDEF0123456789  bin/find.exe\n";
        let entries = parse_sums(text).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(
            entries[0].0,
            "abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789"
        );
        assert_eq!(entries[0].1, "bin/find.exe");
    }

    #[test]
    fn rejects_malformed_lines() {
        assert!(parse_sums("not-a-hash  bin/find.exe\n").is_err());
        assert!(parse_sums("1234567890  bin/find.exe\n").is_err());
    }
}
