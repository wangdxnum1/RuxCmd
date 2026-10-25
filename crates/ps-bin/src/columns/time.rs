use crate::process::Process;

fn format_time_short(t: Option<std::time::SystemTime>) -> String {
    use std::time::UNIX_EPOCH;
    match t {
        Some(s) => {
            let dur = s.duration_since(UNIX_EPOCH).unwrap_or_default();
            let total_min = dur.as_secs() / 60;
            let hh = (total_min / 60) % 24;
            let mm = total_min % 60;
            format!("{hh:02}:{mm:02}")
        }
        None => "?".into(),
    }
}

pub(super) fn etime(p: &Process) -> String {
    let Some(start) = p.start_time else {
        return "?".into();
    };
    let secs = std::time::SystemTime::now()
        .duration_since(start)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let dd = secs / 86400;
    let hh = (secs / 3600) % 24;
    let mm = (secs / 60) % 60;
    let ss = secs % 60;
    if dd > 0 {
        format!("{dd:02}-{hh:02}:{mm:02}:{ss:02}")
    } else {
        format!("{hh:02}:{mm:02}:{ss:02}")
    }
}

pub(super) fn etimes(p: &Process) -> String {
    p.start_time.map_or("?".into(), |s| {
        std::time::SystemTime::now()
            .duration_since(s)
            .map_or("?".into(), |d| d.as_secs().to_string())
    })
}

pub(super) fn times(p: &Process) -> String {
    format!("{}:{}", p.user_time.as_secs(), p.kernel_time.as_secs())
}

pub(super) fn time(p: &Process) -> String {
    let total = (p.kernel_time + p.user_time).as_secs();
    format!(
        "{:02}:{:02}:{:02}",
        total / 3600,
        (total / 60) % 60,
        total % 60
    )
}

pub(super) fn stime(p: &Process) -> String {
    format_time_short(p.start_time)
}

pub(super) fn start(p: &Process) -> String {
    use std::time::UNIX_EPOCH;
    match p.start_time {
        Some(s) => s.duration_since(UNIX_EPOCH).map_or("?".into(), |d| {
            let total_min = d.as_secs() / 60;
            let hh = (total_min / 60) % 24;
            let mm = total_min % 60;
            format!("{hh:02}:{mm:02}")
        }),
        None => "?".into(),
    }
}

pub(super) fn lstart(p: &Process) -> String {
    p.start_time.map_or("?".into(), |s| {
        s.duration_since(std::time::UNIX_EPOCH)
            .map_or("?".into(), |d| format!("{}", d.as_secs()))
    })
}
