use super::{all, lookup, Column};
use crate::process::{Process, ProcessFlags, User};
use std::time::{Duration, UNIX_EPOCH};

fn fixture() -> Process {
    Process {
        pid: 42,
        ppid: 7,
        tgid: 42,
        session_id: 2,
        name: "probe.exe".into(),
        image_path: r"C:\probe.exe".into(),
        cmdline: Some("probe.exe --flag".into()),
        user: User {
            name: "tester".into(),
            sid: "S-1-5-21-42".into(),
        },
        state: 'S',
        priority_class: 0x80,
        nice: -5,
        start_time: None,
        kernel_time: Duration::from_secs(3),
        user_time: Duration::from_secs(7),
        thread_count: 4,
        working_set: 0,
        virtual_size: 4096,
        flags: ProcessFlags::ELEVATED,
    }
}
fn value(name: &str, process: &Process) -> String {
    lookup(name).unwrap().format(process)
}
fn selected(names: &[&str]) -> Vec<&'static Column> {
    names.iter().map(|name| lookup(name).unwrap()).collect()
}

#[test]
fn all_column_metadata_and_values_match_original_fixture() {
    let p = fixture();
    let actual: Vec<_> = all()
        .iter()
        .map(|col| {
            let name = col.name();
            format!(
                "{name}\t{}\t{}\t{}\t{}",
                col.header(),
                col.width(),
                col.align_right(),
                col.format(&p)
            )
        })
        .collect();
    let expected: Vec<_> = include_str!("fixtures/columns.tsv")
        .lines()
        .map(str::to_owned)
        .collect();
    assert_eq!(actual, expected);
}

#[test]
fn system_identity_and_missing_cmdline_keep_existing_fallbacks() {
    let mut p = fixture();
    p.session_id = 0;
    p.cmdline = None;
    assert_eq!(value("uid", &p), "SYSTEM");
    assert_eq!(value("user", &p), "SYSTEM");
    assert_eq!(value("ruser", &p), "tester");
    for name in ["args", "cmd", "command"] {
        assert_eq!(value(name, &p), r"C:\probe.exe");
    }
    for name in ["group", "rgroup", "tty", "tt", "c", "cp"] {
        assert_eq!(value(name, &p), "?");
    }
}

#[test]
fn fixed_start_and_cpu_time_formats_match_existing_columns() {
    let mut p = fixture();
    p.start_time = Some(UNIX_EPOCH + Duration::from_secs(3661));
    assert_eq!(value("start", &p), "01:01");
    assert_eq!(value("stime", &p), "01:01");
    assert_eq!(value("lstart", &p), "3661");
    assert_eq!(value("time", &p), "00:00:10");
    assert_eq!(value("times", &p), "7:3");
}

#[test]
fn sorting_preserves_numeric_order_aliases_and_selected_column_scope() {
    let mut a = fixture();
    a.pid = 1;
    a.nice = 19;
    let mut b = fixture();
    b.pid = 2;
    b.nice = -5;
    let mut c = fixture();
    c.pid = 3;
    c.nice = 10;
    let inputs = vec![c, b, a];
    let spec = crate::sort::parse_sort(&["-pid".into()]);
    let sorted = crate::sort::apply_sort(inputs.clone(), &spec, &selected(&["pid"]));
    assert_eq!(sorted.iter().map(|p| p.pid).collect::<Vec<_>>(), [3, 2, 1]);
    let spec = crate::sort::parse_sort(&["nice".into()]);
    let sorted = crate::sort::apply_sort(inputs.clone(), &spec, &selected(&["ni"]));
    assert_eq!(sorted.iter().map(|p| p.pid).collect::<Vec<_>>(), [2, 3, 1]);
    let sorted = crate::sort::apply_sort(inputs, &spec, &selected(&["pid"]));
    assert_eq!(sorted.iter().map(|p| p.pid).collect::<Vec<_>>(), [1, 2, 3]);
}

#[test]
fn pmem_uses_real_total_physical_memory() {
    // Preserve the original nonzero working-set regression from ps-core.
    let total =
        crate::meminfo::total_physical_memory().expect("total physical memory must be available");
    assert!(total > 0);
    let working_set: u64 = 1_000_000_000;
    let mut p = fixture();
    p.working_set = working_set;
    let out = value("pmem", &p);
    let expected = format!("{:.1}", (working_set as f64 / total as f64) * 100.0);
    assert_eq!(out, expected, "%MEM must use real total physical memory");
    let value: f64 = out.parse().expect("%MEM must parse as a number");
    assert!(value > 0.0 && value <= 100.0);
}
