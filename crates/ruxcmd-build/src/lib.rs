//! Build-only Windows VERSIONINFO support. Resources are linked to binaries,
//! never exported by library dependencies.
use std::{collections::BTreeSet, env, fs, path::PathBuf};

#[derive(Debug)]
struct WindowsVersion {
    parts: [u16; 4],
    prerelease: bool,
}

impl WindowsVersion {
    fn file_version(&self) -> String {
        self.parts.map(|v| v.to_string()).join(".")
    }
}

fn parse_version(value: &str) -> Result<WindowsVersion, String> {
    let version =
        semver::Version::parse(value).map_err(|e| format!("Invalid suite version: {e}"))?;
    let mut parts = [0; 4];
    for (index, part) in [version.major, version.minor, version.patch]
        .into_iter()
        .enumerate()
    {
        parts[index] = u16::try_from(part)
            .map_err(|_| format!("Windows version component exceeds 65535: {value}"))?;
    }
    Ok(WindowsVersion {
        parts,
        prerelease: !version.pre.is_empty(),
    })
}

fn escape_rc(value: &str) -> Result<String, String> {
    if value.chars().any(char::is_control) {
        return Err("Version resource strings cannot contain control characters".into());
    }
    // RC string literals use doubled quotes, rather than C-style \".
    Ok(value.replace('\\', "\\\\").replace('"', "\"\""))
}

fn required(table: &toml::Value, key: &str) -> Result<String, String> {
    let value = table
        .get(key)
        .and_then(toml::Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| format!("Missing version resource field: {key}"))?;
    escape_rc(value)?;
    Ok(value.to_owned())
}

struct Resource {
    name: String,
    rc: String,
}

fn render_resources(workspace: &str, package: &str, debug: bool) -> Result<Vec<Resource>, String> {
    let workspace: toml::Table = workspace
        .parse()
        .map_err(|e| format!("Workspace TOML: {e}"))?;
    let package: toml::Table = package.parse().map_err(|e| format!("Package TOML: {e}"))?;
    let suite = workspace
        .get("workspace")
        .and_then(|v| v.get("metadata"))
        .and_then(|v| v.get("ruxcmd"))
        .ok_or("Missing workspace.metadata.ruxcmd")?;
    let version_string = required(suite, "version")?;
    let version = parse_version(&version_string)?;
    let windows = suite
        .get("windows")
        .ok_or("Missing ruxcmd.windows metadata")?;
    let product = required(windows, "productName")?;
    let company = required(windows, "companyName")?;
    let copyright = required(windows, "copyright")?;
    let metadata = package.get("package").ok_or("Missing package metadata")?;
    let description = required(metadata, "description")?;
    let names = if let Some(bins) = package.get("bin") {
        bins.as_array()
            .ok_or("Expected [[bin]] array")?
            .iter()
            .map(|bin| required(bin, "name"))
            .collect::<Result<Vec<_>, _>>()?
    } else {
        vec![required(metadata, "name")?]
    };
    if names.is_empty() {
        return Err("No binary targets for version resources".into());
    }
    let mut unique = BTreeSet::new();
    let mut resources = Vec::new();
    for name in names {
        if !name
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || c == b'-' || c == b'_')
        {
            return Err(format!("Invalid binary target name: {name}"));
        }
        if !unique.insert(name.to_ascii_lowercase()) {
            return Err(format!("Duplicate binary target: {name}"));
        }
        let flags = u32::from(debug) | (u32::from(version.prerelease) << 1);
        let numeric = version.parts.map(|v| v.to_string()).join(",");
        let fields = [
            ("FileDescription", description.clone()),
            ("CompanyName", company.clone()),
            ("ProductName", product.clone()),
            ("FileVersion", version.file_version()),
            ("ProductVersion", version_string.clone()),
            ("LegalCopyright", copyright.clone()),
            ("OriginalFilename", format!("{name}.exe")),
            ("InternalName", name.clone()),
        ];
        let strings = fields
            .into_iter()
            .map(|(key, value)| {
                Ok(format!(
                    "            VALUE \"{key}\", \"{}\\0\"\n",
                    escape_rc(&value)?
                ))
            })
            .collect::<Result<Vec<_>, String>>()?
            .join("");
        let rc = format!(
            r#"#pragma code_page(65001)
LANGUAGE 0x09, 0x01
1 VERSIONINFO
FILEVERSION {numeric}
PRODUCTVERSION {numeric}
FILEFLAGSMASK 0x3fL
FILEFLAGS {flags}
FILEOS 0x40004L
FILETYPE 0x1L
FILESUBTYPE 0x0L
BEGIN
    BLOCK "StringFileInfo"
    BEGIN
        BLOCK "040904b0"
        BEGIN
{strings}        END
    END
    BLOCK "VarFileInfo"
    BEGIN
        VALUE "Translation", 0x0409, 1200
    END
END
"#
        );
        resources.push(Resource { name, rc });
    }
    Ok(resources)
}

/// Compile version resources for the calling package's named Windows binaries.
/// Missing metadata or a missing/broken Windows resource compiler is fatal.
pub fn embed_version() {
    if env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }
    let package_dir =
        PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").expect("Cargo manifest directory"));
    let root = package_dir
        .parent()
        .and_then(|p| p.parent())
        .expect("crates/package workspace layout");
    let workspace_manifest = root.join("Cargo.toml");
    let package_manifest = package_dir.join("Cargo.toml");
    for path in [
        &workspace_manifest,
        &package_manifest,
        &root.join("build/windows-version.rs"),
    ] {
        println!("cargo:rerun-if-changed={}", path.display());
    }
    let workspace = fs::read_to_string(&workspace_manifest).expect("Read suite manifest");
    let package = fs::read_to_string(&package_manifest).expect("Read package manifest");
    let debug = env::var("PROFILE").as_deref() == Ok("debug");
    let resources =
        render_resources(&workspace, &package, debug).expect("Windows version metadata");
    let output = PathBuf::from(env::var_os("OUT_DIR").expect("Cargo output directory"));
    for resource in resources {
        let path = output.join(format!("{}-version.rc", resource.name));
        fs::write(&path, resource.rc).expect("Write Windows version resource");
        embed_resource::compile_for(&path, [&resource.name], embed_resource::NONE)
            .manifest_required()
            .expect("Compile Windows version resource (Windows SDK required)");
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    const WORKSPACE: &str = r#"
[workspace.metadata.ruxcmd]
version = "0.3.1"
[workspace.metadata.ruxcmd.windows]
productName = "RuxCmd"
companyName = "Tim and RuxCmd contributors"
copyright = "Copyright (c) 2026 Tim and RuxCmd contributors"
"#;
    const PACKAGE: &str = r#"
[package]
name = "ps-bin"
description = "Display running processes"
[[bin]]
name = "ps"
path = "src/main.rs"
"#;

    #[test]
    fn suite_version_maps_to_windows_words_and_prerelease_flag() {
        let version = parse_version("0.3.1-beta.2+build.7").unwrap();
        assert_eq!(version.parts, [0, 3, 1, 0]);
        assert!(version.prerelease);
        assert_eq!(version.file_version(), "0.3.1.0");
    }

    #[test]
    fn invalid_or_out_of_range_versions_fail() {
        for version in [
            "bad",
            "1.2",
            "1.2.3.4",
            "65536.1.2",
            "1.65536.2",
            "1.2.65536",
        ] {
            assert!(parse_version(version).is_err(), "{version}");
        }
        assert_eq!(
            parse_version("65535.65535.65535").unwrap().parts,
            [65535, 65535, 65535, 0]
        );
    }

    #[test]
    fn alias_uses_executable_identity_and_suite_version() {
        let resources = render_resources(WORKSPACE, PACKAGE, false).unwrap();
        assert_eq!(resources.len(), 1);
        assert_eq!(resources[0].name, "ps");
        let rc = &resources[0].rc;
        assert!(rc.contains("FILEVERSION 0,3,1,0"));
        assert!(rc.contains("PRODUCTVERSION 0,3,1,0"));
        assert!(rc.contains("\"FileVersion\", \"0.3.1.0\\0\""));
        assert!(rc.contains("\"ProductVersion\", \"0.3.1\\0\""));
        assert!(rc.contains("\"OriginalFilename\", \"ps.exe\\0\""));
        assert!(!rc.contains("ps-bin.exe"));
    }

    #[test]
    fn implicit_binary_uses_its_own_identity_even_with_a_library() {
        let package = "[package]\nname='vdir'\ndescription='List directory contents in long format'\n[lib]\npath='src/lib.rs'";
        let rc = &render_resources(WORKSPACE, package, false).unwrap()[0].rc;
        assert!(rc.contains("\"InternalName\", \"vdir\\0\""));
        assert!(rc.contains("\"OriginalFilename\", \"vdir.exe\\0\""));
    }

    #[test]
    fn unicode_quotes_and_backslashes_are_encoded_in_rc() {
        assert_eq!(
            escape_rc("列出 \"files\" at C:\\temp").unwrap(),
            r#"列出 ""files"" at C:\\temp"#
        );
        assert!(escape_rc("bad\0value").is_err());
        assert!(escape_rc("bad\nvalue").is_err());
        let rc = &render_resources(WORKSPACE, PACKAGE, false).unwrap()[0].rc;
        assert!(rc.contains("#pragma code_page(65001)"));
        assert!(rc.contains("\"Translation\", 0x0409, 1200"));
    }

    #[test]
    fn missing_fields_and_duplicate_targets_fail() {
        assert!(render_resources(WORKSPACE, "[package]\nname='empty'", false).is_err());
        assert!(render_resources(&WORKSPACE.replace("RuxCmd\"", "\""), PACKAGE, false).is_err());
        let duplicate = format!("{PACKAGE}\n[[bin]]\nname='ps'\npath='other.rs'");
        assert!(render_resources(WORKSPACE, &duplicate, false).is_err());
    }

    #[test]
    fn debug_and_prerelease_flags_are_combined() {
        let workspace = WORKSPACE.replace("0.3.1", "0.3.2-rc.1");
        let rc = &render_resources(&workspace, PACKAGE, true).unwrap()[0].rc;
        assert!(rc.contains("FILEFLAGS 3"));
        assert!(rc.contains("\"ProductVersion\", \"0.3.2-rc.1\\0\""));
    }
}
