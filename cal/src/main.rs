mod cli;

use chrono::{Datelike, Local, Months, NaiveDate};
use clap::Parser;

fn main() {
    let args = cli::Args::parse();
    let today = Local::now().date_naive();
    let current_year = today.year();
    let current_month = today.month();

    let target_month = args.month.unwrap_or(current_month);
    let target_year = args.year_arg.unwrap_or(current_year);

    if args.year {
        print_year(target_year, args.julian);
    } else if args.three_months {
        print_three_months(target_year, target_month, args.julian);
    } else {
        print_month(target_year, target_month, args.julian);
    }
}

fn print_month(year: i32, month: u32, julian: bool) {
    let first_day = NaiveDate::from_ymd_opt(year, month, 1).unwrap();
    let last_day = first_day
        .checked_add_months(Months::new(1))
        .unwrap()
        .pred();
    let days_in_month = last_day.day();
    let first_weekday = first_day.weekday().num_days_from_sunday();

    println!("   {} {}", month_name(month), year);
    println!(" Su Mo Tu We Th Fr Sa");

    for _ in 0..first_weekday {
        print!("   ");
    }

    for day in 1..=days_in_month {
        if julian {
            let date = NaiveDate::from_ymd_opt(year, month, day).unwrap();
            let julian_day = date.ordinal();
            print!(" {:>2}", julian_day);
        } else {
            print!(" {:>2}", day);
        }
        if (first_weekday + day) % 7 == 0 {
            println!();
        }
    }
    if (first_weekday + days_in_month) % 7 != 0 {
        println!();
    }
    println!();
}

fn print_three_months(year: i32, month: u32, julian: bool) {
    let prev_month = if month == 1 { 12 } else { month - 1 };
    let prev_year = if month == 1 { year - 1 } else { year };
    let next_month = if month == 12 { 1 } else { month + 1 };
    let next_year = if month == 12 { year + 1 } else { year };

    let months = [(prev_year, prev_month), (year, month), (next_year, next_month)];
    
    for (y, m) in &months {
        print!("      {} {}      ", month_name(*m), y);
    }
    println!();
    
    for _ in 0..3 {
        print!(" Su Mo Tu We Th Fr Sa   ");
    }
    println!();

    let mut month_data: Vec<Vec<String>> = Vec::new();
    for (y, m) in &months {
        let first_day = NaiveDate::from_ymd_opt(*y, *m, 1).unwrap();
        let last_day = first_day
            .checked_add_months(Months::new(1))
            .unwrap()
            .pred();
        let days_in_month = last_day.day();
        let first_weekday = first_day.weekday().num_days_from_sunday();

        let mut row = Vec::new();
        for _ in 0..first_weekday {
            row.push("   ".to_string());
        }
        for day in 1..=days_in_month {
            if julian {
                let date = NaiveDate::from_ymd_opt(*y, *m, day).unwrap();
                let julian_day = date.ordinal();
                row.push(format!(" {:>2}", julian_day));
            } else {
                row.push(format!(" {:>2}", day));
            }
        }
        while row.len() < 42 {
            row.push("   ".to_string());
        }
        month_data.push(row);
    }

    for week in 0..6 {
        for m in 0..3usize {
            for day in 0..7usize {
                let idx = week * 7 + day;
                if idx < month_data[m].len() {
                    print!("{}", month_data[m][idx]);
                }
            }
            print!("   ");
        }
        println!();
    }
    println!();
}

fn print_year(year: i32, julian: bool) {
    let months_per_row = 3usize;
    let rows = 4usize;

    for row in 0..rows {
        let start_month = (row * months_per_row + 1) as u32;
        for m in start_month..start_month + months_per_row as u32 {
            print!("      {} {}      ", month_name(m), year);
        }
        println!();

        for _ in 0..months_per_row {
            print!(" Su Mo Tu We Th Fr Sa   ");
        }
        println!();

        let mut month_data: Vec<Vec<String>> = Vec::new();
        for m in start_month..start_month + months_per_row as u32 {
            let first_day = NaiveDate::from_ymd_opt(year, m, 1).unwrap();
            let last_day = first_day
                .checked_add_months(Months::new(1))
                .unwrap()
                .pred();
            let days_in_month = last_day.day();
            let first_weekday = first_day.weekday().num_days_from_sunday();

            let mut row_data = Vec::new();
            for _ in 0..first_weekday {
                row_data.push("   ".to_string());
            }
            for day in 1..=days_in_month {
                if julian {
                    let date = NaiveDate::from_ymd_opt(year, m, day).unwrap();
                    let julian_day = date.ordinal();
                    row_data.push(format!(" {:>2}", julian_day));
                } else {
                    row_data.push(format!(" {:>2}", day));
                }
            }
            while row_data.len() < 42 {
                row_data.push("   ".to_string());
            }
            month_data.push(row_data);
        }

        for week in 0..6usize {
            for m in 0..months_per_row {
                for day in 0..7usize {
                    let idx = week * 7 + day;
                    if idx < month_data[m].len() {
                        print!("{}", month_data[m][idx]);
                    }
                }
                print!("   ");
            }
            println!();
        }
        println!();
    }
}

fn month_name(month: u32) -> &'static str {
    match month {
        1 => "January",
        2 => "February",
        3 => "March",
        4 => "April",
        5 => "May",
        6 => "June",
        7 => "July",
        8 => "August",
        9 => "September",
        10 => "October",
        11 => "November",
        12 => "December",
        _ => "",
    }
}
