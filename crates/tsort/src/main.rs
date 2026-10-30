use clap::Parser;
use std::collections::{BTreeMap, BTreeSet};
use std::io::BufRead;

/// Topologically sort a directed graph of whitespace-separated pairs -
/// a Windows port of Linux tsort.
#[derive(Parser, Debug)]
#[command(name = "tsort", version, about)]
pub struct Args {
    /// Files describing edges (default: standard input)
    #[arg(value_name = "FILE")]
    pub files: Vec<String>,
}

fn main() {
    let args = Args::parse();
    let mut tokens: Vec<String> = Vec::new();

    let mut input_name = "-".to_string();
    if args.files.is_empty() {
        if let Err(e) = read_tokens(&mut std::io::stdin().lock(), &mut tokens) {
            eprintln!("tsort: {}: error reading stdin: {}", input_name, e);
            std::process::exit(1);
        }
    } else {
        for file in &args.files {
            input_name = file.clone();
            if file == "-" {
                if let Err(e) = read_tokens(&mut std::io::stdin().lock(), &mut tokens) {
                    eprintln!("tsort: {}: error reading stdin: {}", input_name, e);
                    std::process::exit(1);
                }
                continue;
            }
            match std::fs::File::open(file) {
                Ok(handle) => {
                    if let Err(e) = read_tokens(&mut std::io::BufReader::new(handle), &mut tokens) {
                        eprintln!("tsort: {}: {}", file, e);
                        std::process::exit(1);
                    }
                }
                Err(e) => {
                    eprintln!("tsort: {}: {}", file, e);
                    std::process::exit(1);
                }
            }
        }
    }

    if !tokens.len().is_multiple_of(2) {
        eprintln!(
            "tsort: {}: input contains an odd number of tokens",
            input_name
        );
        std::process::exit(1);
    }

    let stdout = std::io::stdout();
    let mut out = stdout.lock();
    use std::io::Write;
    // GNU reports loops on stderr, still emits the loop members on stdout,
    // and exits 1 after the order.
    let (order, loops) = toposort(&tokens);
    for node in &order {
        let _ = writeln!(out, "{}", node);
    }
    if !loops.is_empty() {
        eprintln!("tsort: {}: input contains a loop:", input_name);
        for node in &loops {
            eprintln!("tsort: {}", node);
            let _ = writeln!(out, "{}", node);
        }
        std::process::exit(1);
    }
}

fn read_tokens(reader: &mut impl BufRead, tokens: &mut Vec<String>) -> std::io::Result<()> {
    let mut line = String::new();
    loop {
        line.clear();
        let n = reader.read_line(&mut line)?;
        if n == 0 {
            return Ok(());
        }
        tokens.extend(line.split_whitespace().map(str::to_string));
    }
}

/// Kahn's algorithm with GNU's tie-breaking of first-seen order. Returns
/// the emitted order plus the nodes of the first remaining loop (empty when
/// the graph is acyclic).
fn toposort(tokens: &[String]) -> (Vec<String>, Vec<String>) {
    let mut successors: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let mut indegree: BTreeMap<String, usize> = BTreeMap::new();
    let mut seen: Vec<String> = Vec::new();

    for pair in tokens.chunks(2) {
        for node in [&pair[0], &pair[1]] {
            if !indegree.contains_key(node) {
                indegree.insert(node.clone(), 0);
                successors.entry(node.clone()).or_default();
                seen.push(node.clone());
            }
        }
        let from = &pair[0];
        let to = &pair[1];
        if successors
            .entry(from.clone())
            .or_default()
            .insert(to.clone())
        {
            *indegree.get_mut(to).unwrap() += 1;
        }
    }

    let mut first_seen: BTreeMap<&str, usize> = BTreeMap::new();
    for (i, node) in seen.iter().enumerate() {
        first_seen.insert(node, i);
    }

    let mut ready: Vec<String> = indegree
        .iter()
        .filter(|(_, deg)| **deg == 0)
        .map(|(node, _)| node.clone())
        .collect();
    ready.sort_by_key(|node| first_seen[node.as_str()]);

    let mut order: Vec<String> = Vec::new();
    while let Some(node) = ready.first().cloned() {
        ready.remove(0);
        order.push(node.clone());
        if let Some(targets) = successors.get(&node) {
            for target in targets.clone() {
                let entry = indegree.get_mut(&target).unwrap();
                *entry -= 1;
                if *entry == 0 {
                    ready.push(target);
                }
            }
        }
        ready.sort_by_key(|n| first_seen[n.as_str()]);
    }

    if order.len() == indegree.len() {
        return (order, Vec::new());
    }

    // Walk the successor chain of the first unemitted node until it
    // revisits; that cycle is reported the way GNU does.
    let emitted: BTreeSet<&String> = order.iter().collect();
    let start = indegree
        .keys()
        .find(|node| !emitted.contains(node))
        .cloned()
        .unwrap_or_default();
    let mut loop_nodes: Vec<String> = Vec::new();
    let mut current = start;
    loop {
        loop_nodes.push(current.clone());
        let next = successors
            .get(&current)
            .and_then(|targets| targets.iter().next())
            .cloned()
            .unwrap_or_default();
        if next.is_empty() || loop_nodes.contains(&next) {
            break;
        }
        current = next;
    }
    (order, loop_nodes)
}
