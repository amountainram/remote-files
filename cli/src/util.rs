// use crate::client::StatEntry;
// use opendal::EntryMode;
// use prettytable::{format, row, Table};

// fn parse_content_length(input: &str, raw: bool) -> String {
//     if raw || input.is_empty() {
//         return input.to_string();
//     }

//     match input.len() {
//         1..=3 => format!("{input}B"),
//         4..=6 => {
//             let mut kilobytes = input.to_string();
//             kilobytes.truncate(input.len() - 2);
//             let mut kilobytes: Vec<char> = kilobytes.chars().collect();
//             kilobytes.insert(kilobytes.len() - 1, '.');
//             let kilobytes: String = kilobytes.into_iter().collect();
//             format!("{}kB", kilobytes)
//         }
//         _ => {
//             let mut megabytes = input.to_string();
//             megabytes.truncate(input.len() - 5);
//             let mut megabytes: Vec<char> = megabytes.chars().collect();
//             megabytes.insert(megabytes.len() - 1, '.');
//             let megabytes: String = megabytes.into_iter().collect();
//             format!("{}MB", megabytes)
//         }
//     }
// }

// pub fn log_profiles_table(mut items: Vec<&String>, current: Option<&str>) {
//     let mut table = Table::new();

//     table.set_format(*format::consts::FORMAT_NO_LINESEP_WITH_TITLE);

//     table.set_titles(row![Fgb->"", Fgb->"name"]);
//     items.sort();
//     for item in items.iter_mut() {
//         let is_current = current.is_some_and(|val| val == item.as_str());
//         if is_current {
//             table.add_row(row![Fgcb->"👉", Fbb->item]);
//         } else {
//             table.add_row(row!["", Fbb->item]);
//         }
//     }

//     table.print_tty(true).unwrap();
// }

// pub fn log_files_table(items: &[StatEntry], raw: bool, should_paginate: bool) {
//     let mut table = Table::new();

//     table.set_format(*format::consts::FORMAT_NO_LINESEP_WITH_TITLE);

//     table.set_titles(row!["", Fgb->"name", Fgb->"content-type", Fgb->"size", Fgb->"type"]);
//     for (line, item) in items.iter().enumerate() {
//         let line = line + 1;
//         match item.3 {
//             EntryMode::FILE => {
//                 table.add_row(row![Fw-> line, Fw->item.0,Fbb->item.1,Fbb->parse_content_length(&item.2, raw),Fbb->"file"]);
//             }
//             EntryMode::DIR => {
//                 table.add_row(row![Fw-> line, Fm->item.0, "", "", Fmb->"dir"]);
//             }
//             EntryMode::Unknown => {}
//         };
//     }

//     if !should_paginate {
//         table.add_row(row![Fw -> "", Fm-> "...", "", "", ""]);
//     }

//     table.print_tty(true).unwrap();
// }

// pub enum NextAction {
//     Quit,
//     Next,
//     Print(usize),
// }

// pub async fn what_next() -> NextAction {
//     let mut input = String::new();
//     let _ = std::io::stdin().read_line(&mut input);
//     let trimmed_len = input.len() - 1;

//     if let Ok(idx) = &input[..trimmed_len].parse::<usize>() {
//         return NextAction::Print(*idx);
//     }

//     match (trimmed_len, &input.as_bytes()[..trimmed_len]) {
//         (1, [b'q']) => NextAction::Quit,
//         _ => NextAction::Next,
//     }
// }

// #[macro_export]
// macro_rules! opendal_builder {
//     ($builder:expr, $( $opt:expr => $method:ident ),* ) => {{
//         let builder = $builder;
//         $(
//             let builder = if let Some(value) = $opt {
//                 builder.$method(value)
//             } else {
//                 builder
//             };
//         )*
//         builder
//     }};
// }
use anyhow::{anyhow, Result};
use colored::Colorize;
use std::collections::HashSet;
use tabled::{
    settings::{object::Segment, Alignment, Settings},
    Table, Tabled,
};

static RF_ICON: &str = "🪣 ";

pub fn welcome() {
    println!(
        "\n{}{} '{}'\n",
        "Welcome to ".magenta(),
        RF_ICON,
        "remote-files".bold().magenta()
    );
    println!("{}\n", "-".repeat(50))
}

pub fn msg_ok<S>(text: S)
where
    S: AsRef<str>,
{
    println!("{}", text.as_ref().green());
}

#[derive(Tabled)]
pub struct ProfileInfo<'a> {
    #[tabled(order = 1)]
    pub name: &'a str,
    #[tabled(order = 0, display_with("display_current"))]
    pub current: bool,
}

fn display_current(current: &bool) -> &str {
    if *current {
        "👉"
    } else {
        ""
    }
}

pub fn list_profiles(profiles: &[ProfileInfo<'_>]) -> String {
    let mut table = Table::new(profiles);
    table.modify(
        Segment::all(),
        Settings::new(Alignment::center(), Alignment::center()),
    );
    table.to_string()
}

pub fn print_table(table: &str) {
    println!("\n{table}\n");
}

pub fn get_profile<'a>(
    args_profile: Option<&'a str>,
    current: Option<&'a str>,
    cfg: HashSet<&'a str>,
) -> Result<&'a str> {
    // get current profile if any
    let profile = args_profile
        .or(current)
        .ok_or(anyhow!("no profile selected"))?;

    cfg.get(profile)
        .copied()
        .ok_or(anyhow!("no profile '{profile}' found"))
}
