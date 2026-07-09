use std::io::Write;

use ps_core::column::{Column, FormatCtx, ProcessSample};
use ps_core::process::Process;

pub fn render_table<W: Write>(
    out: &mut W,
    procs: &[Process],
    cols: &[Box<dyn Column>],
    no_headers: bool,
) -> std::io::Result<()> {
    let ctx = FormatCtx {
        now: std::time::Instant::now(),
        prev: None,
    };
    if !no_headers {
        let headers: Vec<String> = cols
            .iter()
            .map(|c| pad(c.header(), c.width(), c.align_right()))
            .collect();
        writeln!(out, "{}", headers.join(" "))?;
    }
    for p in procs {
        let row: Vec<String> = cols
            .iter()
            .map(|c| {
                let s = c.format(p, &ctx);
                pad(&s, c.width(), c.align_right())
            })
            .collect();
        writeln!(out, "{}", row.join(" "))?;
    }
    Ok(())
}

pub fn render_forest<W: Write>(
    out: &mut W,
    forest: &ps_core::tree::Forest,
    cols: &[Box<dyn Column>],
    no_headers: bool,
    ascii_mode: bool,
) -> std::io::Result<()> {
    let ctx = FormatCtx {
        now: std::time::Instant::now(),
        prev: None,
    };
    if !no_headers {
        let mut headers: Vec<String> = vec![pad("", cols[0].width(), false)];
        for c in cols {
            headers.push(pad(c.header(), c.width(), c.align_right()));
        }
        writeln!(out, "{}", headers.join(" "))?;
    }
    let (mid, end, bar) = if ascii_mode {
        ("+--", "`--", "|  ")
    } else {
        ("├──", "└──", "│  ")
    };
    for (i, root) in forest.roots.iter().enumerate() {
        let last = i + 1 == forest.roots.len();
        render_node(out, root, "", last, cols, &ctx, mid, end, bar, no_headers)?;
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn render_node<W: Write>(
    out: &mut W,
    node: &ps_core::tree::Node,
    prefix: &str,
    last: bool,
    cols: &[Box<dyn Column>],
    ctx: &FormatCtx,
    mid: &str,
    end: &str,
    bar: &str,
    _no_headers: bool,
) -> std::io::Result<()> {
    let symbol = if last { end } else { mid };
    let row: Vec<String> = std::iter::once(format!("{prefix}{symbol}"))
        .chain(cols.iter().map(|c| {
            let s = c.format(&node.proc, ctx);
            pad(&s, c.width(), c.align_right())
        }))
        .collect();
    writeln!(out, "{}", row.join(" "))?;
    let next_prefix = format!("{prefix}{}", if last { "   " } else { bar });
    for (i, child) in node.children.iter().enumerate() {
        let child_last = i + 1 == node.children.len();
        render_node(
            out,
            child,
            &next_prefix,
            child_last,
            cols,
            ctx,
            mid,
            end,
            bar,
            _no_headers,
        )?;
    }
    Ok(())
}

fn pad(s: &str, w: usize, right: bool) -> String {
    let len = display_width(s);
    if len >= w {
        truncate(s, w)
    } else if right {
        format!("{}{}", " ".repeat(w - len), s)
    } else {
        format!("{}{}", s, " ".repeat(w - len))
    }
}

fn display_width(s: &str) -> usize {
    s.chars().count()
}

fn truncate(s: &str, w: usize) -> String {
    s.chars().take(w).collect()
}

#[allow(dead_code)]
pub fn make_sample(p: &Process) -> ProcessSample {
    ProcessSample {
        pid: p.pid,
        cpu_total: p.kernel_time + p.user_time,
        at: std::time::Instant::now(),
    }
}
