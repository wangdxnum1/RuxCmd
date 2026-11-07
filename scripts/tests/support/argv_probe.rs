use std::io::{Read, Write};

fn main() {
    let mut args = std::env::args().skip(1);
    let log = args.next().expect("log path");
    let mode = args.next().expect("exit mode");
    let mut items: Vec<String> = args.collect();
    if mode == "stdin" {
        let mut text = String::new();
        std::io::stdin().read_to_string(&mut text).unwrap();
        items.push(format!("stdin:{text}"));
    }
    let mut record = Vec::new();
    record.extend_from_slice(&(items.len() as u32).to_le_bytes());
    for item in &items {
        record.extend_from_slice(&(item.len() as u32).to_le_bytes());
        record.extend_from_slice(item.as_bytes());
    }
    std::fs::OpenOptions::new().create(true).append(true).open(log).unwrap()
        .write_all(&record).unwrap();
    let code = if mode == "by-item" {
        match items.first().map(String::as_str) {
            Some("stop") => 255,
            Some("fail") => 7,
            _ => 0,
        }
    } else { mode.parse::<i32>().unwrap_or(0) };
    std::process::exit(code);
}
