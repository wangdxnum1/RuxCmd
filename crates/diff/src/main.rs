mod cli;

use clap::Parser;
use cli::Cli;
use std::fs::read_to_string;
use std::process;

fn main() {
    let cli = Cli::parse();

    if cli.version {
        println!("diff 0.1.0");
        process::exit(0);
    }

    let file1 = match &cli.file1 {
        Some(f) => f,
        None => {
            eprintln!("error: the following required arguments were not provided:");
            eprintln!("  <FILE1>");
            eprintln!("  <FILE2>");
            eprintln!("");
            eprintln!("Usage: diff [OPTIONS] <FILE1> <FILE2>");
            process::exit(1);
        }
    };

    let file2 = match &cli.file2 {
        Some(f) => f,
        None => {
            eprintln!("error: the following required arguments were not provided:");
            eprintln!("  <FILE2>");
            eprintln!("");
            eprintln!("Usage: diff [OPTIONS] <FILE1> <FILE2>");
            process::exit(1);
        }
    };

    let file1_content = match read_to_string(file1) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Error reading {}: {}", file1, e);
            process::exit(1);
        }
    };

    let file2_content = match read_to_string(file2) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Error reading {}: {}", file2, e);
            process::exit(1);
        }
    };

    let lines1: Vec<String> = file1_content.lines().map(|s| s.to_string()).collect();
    let lines2: Vec<String> = file2_content.lines().map(|s| s.to_string()).collect();

    if cli.unified {
        print_unified_diff(file1, file2, &lines1, &lines2, &cli);
    } else {
        print_normal_diff(&lines1, &lines2, &cli);
    }
}

fn normalize_line(line: &str, cli: &Cli) -> String {
    let mut result = line.to_string();

    if cli.ignore_case {
        result = result.to_lowercase();
    }

    if cli.ignore_whitespace {
        result = result.chars().filter(|c| !c.is_whitespace()).collect();
    }

    result
}

fn print_normal_diff(lines1: &[String], lines2: &[String], cli: &Cli) {
    let n = lines1.len();
    let m = lines2.len();

    let mut dp = vec![vec![0; m + 1]; n + 1];

    for i in 1..=n {
        for j in 1..=m {
            let line1 = normalize_line(&lines1[i - 1], &cli);
            let line2 = normalize_line(&lines2[j - 1], &cli);

            let skip1 = cli.ignore_blank_lines && lines1[i - 1].trim().is_empty();
            let skip2 = cli.ignore_blank_lines && lines2[j - 1].trim().is_empty();

            if line1 == line2 {
                dp[i][j] = dp[i - 1][j - 1] + 1;
            } else if skip1 || skip2 {
                dp[i][j] = dp[i - 1][j - 1];
            } else {
                dp[i][j] = dp[i - 1][j].max(dp[i][j - 1]);
            }
        }
    }

    let mut i = n;
    let mut j = m;
    let mut diffs: Vec<(i32, i32, String, String)> = Vec::new();

    while i > 0 || j > 0 {
        if i > 0 && j > 0 {
            let line1 = normalize_line(&lines1[i - 1], &cli);
            let line2 = normalize_line(&lines2[j - 1], &cli);

            let skip1 = cli.ignore_blank_lines && lines1[i - 1].trim().is_empty();
            let skip2 = cli.ignore_blank_lines && lines2[j - 1].trim().is_empty();

            if line1 == line2 || (skip1 && skip2) {
                i -= 1;
                j -= 1;
            } else if skip1 {
                i -= 1;
            } else if skip2 {
                j -= 1;
            } else if dp[i - 1][j] > dp[i][j - 1] {
                diffs.push((i as i32, 0, lines1[i - 1].clone(), String::new()));
                i -= 1;
            } else {
                diffs.push((0, j as i32, String::new(), lines2[j - 1].clone()));
                j -= 1;
            }
        } else if i > 0 {
            diffs.push((i as i32, 0, lines1[i - 1].clone(), String::new()));
            i -= 1;
        } else {
            diffs.push((0, j as i32, String::new(), lines2[j - 1].clone()));
            j -= 1;
        }
    }

    diffs.reverse();

    let mut i = 1;
    let mut j = 1;

    for diff in &diffs {
        if diff.0 > 0 && diff.1 == 0 {
            println!("{}d{}", i, j);
            println!("< {}", diff.2);
            i += 1;
        } else if diff.0 == 0 && diff.1 > 0 {
            println!("{}a{}", i - 1, j);
            println!("> {}", diff.3);
            j += 1;
        } else if diff.0 > 0 && diff.1 > 0 {
            println!("{}c{}", i, j);
            println!("< {}", diff.2);
            println!("---");
            println!("> {}", diff.3);
            i += 1;
            j += 1;
        } else {
            i += 1;
            j += 1;
        }
    }
}

fn print_unified_diff(file1: &str, file2: &str, lines1: &[String], lines2: &[String], cli: &Cli) {
    let n = lines1.len();
    let m = lines2.len();

    let mut dp = vec![vec![0; m + 1]; n + 1];

    for i in 1..=n {
        for j in 1..=m {
            let line1 = normalize_line(&lines1[i - 1], &cli);
            let line2 = normalize_line(&lines2[j - 1], &cli);

            let skip1 = cli.ignore_blank_lines && lines1[i - 1].trim().is_empty();
            let skip2 = cli.ignore_blank_lines && lines2[j - 1].trim().is_empty();

            if line1 == line2 {
                dp[i][j] = dp[i - 1][j - 1] + 1;
            } else if skip1 || skip2 {
                dp[i][j] = dp[i - 1][j - 1];
            } else {
                dp[i][j] = dp[i - 1][j].max(dp[i][j - 1]);
            }
        }
    }

    let mut i = n;
    let mut j = m;
    let mut diffs: Vec<(usize, usize, bool, String)> = Vec::new();

    while i > 0 || j > 0 {
        if i > 0 && j > 0 {
            let line1 = normalize_line(&lines1[i - 1], &cli);
            let line2 = normalize_line(&lines2[j - 1], &cli);

            let skip1 = cli.ignore_blank_lines && lines1[i - 1].trim().is_empty();
            let skip2 = cli.ignore_blank_lines && lines2[j - 1].trim().is_empty();

            if line1 == line2 || (skip1 && skip2) {
                diffs.push((i, j, true, lines1[i - 1].clone()));
                i -= 1;
                j -= 1;
            } else if skip1 {
                i -= 1;
            } else if skip2 {
                j -= 1;
            } else if dp[i - 1][j] > dp[i][j - 1] {
                diffs.push((i, 0, false, lines1[i - 1].clone()));
                i -= 1;
            } else {
                diffs.push((0, j, false, lines2[j - 1].clone()));
                j -= 1;
            }
        } else if i > 0 {
            diffs.push((i, 0, false, lines1[i - 1].clone()));
            i -= 1;
        } else {
            diffs.push((0, j, false, lines2[j - 1].clone()));
            j -= 1;
        }
    }

    diffs.reverse();

    println!("--- {}", file1);
    println!("+++ {}", file2);

    let mut i = 1;
    let mut j = 1;
    let mut in_hunk = false;
    let mut hunk_start = 0;
    let mut hunk_lines1 = 0;
    let mut hunk_lines2 = 0;
    let mut hunk_content: Vec<String> = Vec::new();

    for diff in &diffs {
        if diff.2 {
            if in_hunk {
                hunk_lines1 += 1;
                hunk_lines2 += 1;
                hunk_content.push(format!(" {}", diff.3));
            }
            i += 1;
            j += 1;
        } else {
            if !in_hunk {
                in_hunk = true;
                hunk_start = i.min(j);
                hunk_lines1 = 0;
                hunk_lines2 = 0;
                hunk_content.clear();
            }

            if diff.0 > 0 {
                hunk_lines1 += 1;
                hunk_content.push(format!("-{}", diff.3));
                i += 1;
            } else {
                hunk_lines2 += 1;
                hunk_content.push(format!("+{}", diff.3));
                j += 1;
            }
        }
    }

    if in_hunk {
        print_hunk(hunk_start, hunk_lines1, hunk_lines2, &hunk_content);
    }
}

fn print_hunk(start: usize, lines1: usize, lines2: usize, content: &[String]) {
    if lines1 == 0 {
        println!("@@ -{} +{},{} @@", start, start, lines2);
    } else if lines2 == 0 {
        println!("@@ -{},{} +{} @@", start, lines1, start);
    } else {
        println!("@@ -{},{} +{},{} @@", start, lines1, start, lines2);
    }

    for line in content {
        println!("{}", line);
    }
}
