#![allow(dead_code)]
use std::path::PathBuf;

pub struct Probe {
    pub dir: tempfile::TempDir,
    pub exe: PathBuf,
    pub log: PathBuf,
}

impl Probe {
    pub fn new() -> Self {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("probe.rs");
        let exe = dir.path().join("argument probe.exe");
        let log = dir.path().join("records.bin");
        std::fs::write(&source, include_str!("argv_probe.rs")).unwrap();
        let compiler = std::env::var_os("RUSTC").unwrap_or_else(|| "rustc".into());
        let result = std::process::Command::new(compiler)
            .args(["--edition=2021", "--crate-name=batch_probe"])
            .arg(source)
            .arg("-o")
            .arg(&exe)
            .output()
            .unwrap();
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        Self { dir, exe, log }
    }

    pub fn prefix(&self, mode: &str) -> Vec<String> {
        vec![
            self.exe.display().to_string(),
            self.log.display().to_string(),
            mode.into(),
        ]
    }

    pub fn records(&self) -> Vec<Vec<String>> {
        let data = std::fs::read(&self.log).unwrap_or_default();
        let mut cursor = 0;
        let mut records = Vec::new();
        fn number(data: &[u8], cursor: &mut usize) -> usize {
            let value = u32::from_le_bytes(data[*cursor..*cursor + 4].try_into().unwrap());
            *cursor += 4;
            value as usize
        }
        while cursor < data.len() {
            let count = number(&data, &mut cursor);
            let mut args = Vec::new();
            for _ in 0..count {
                let len = number(&data, &mut cursor);
                args.push(String::from_utf8(data[cursor..cursor + len].to_vec()).unwrap());
                cursor += len;
            }
            records.push(args);
        }
        records
    }
}
