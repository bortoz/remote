use clap::{App, Arg, SubCommand};
use core::OlinfoClient;
use crossterm;
use rand::Rng;
use std::process::Command;
use std::thread;
use std::time::Duration;
use worker::{Worker, WorkerStatus};

const ROWS: u8 = 4;
const COLUMNS: u8 = 4;

fn parse_target(arg: &str) -> Option<(u8, u8)> {
    let chars: Vec<char> = arg.chars().collect();
    if chars.len() != 2 || !('1' <= chars[1] && chars[1] <= ('1' as u8 + COLUMNS) as char) {
        None
    } else if 'a' <= chars[0] && chars[0] <= ('a' as u8 + ROWS) as char {
        Some((chars[0] as u8 - 96, chars[1] as u8 - 48))
    } else if 'A' <= chars[0] && chars[0] <= ('A' as u8 + ROWS) as char {
        Some((chars[0] as u8 - 64, chars[1] as u8 - 48))
    } else {
        None
    }
}

fn parse_targets(arg: &str) -> Option<Vec<(u8, u8)>> {
    if arg == "all" {
        let mut targets = Vec::new();
        for r in 1..=ROWS {
            for c in 1..=COLUMNS {
                targets.push((r, c));
            }
        }
        Some(targets)
    } else if arg == "rand" {
        let mut rng = rand::thread_rng();
        Some(vec![(
            rng.gen_range(1, ROWS + 1),
            rng.gen_range(1, COLUMNS + 1),
        )])
    } else if let Some(t) = parse_target(arg) {
        Some(vec![t])
    } else {
        None
    }
}

fn main() {
    let matches = App::new("remote")
        .version("1.0.0")
        .arg(
            Arg::with_name("target")
                .short("t")
                .long("target")
                .value_name("TARGET")
                .help("type of target [possible values: all, rand, <TARGET>] [default: all]")
                .takes_value(true)
                .default_value("all")
                .hide_default_value(true),
        )
        .subcommand(
            SubCommand::with_name("run")
                .about("Runs command remotely")
                .arg(
                    Arg::with_name("command")
                        .required(true)
                        .index(1)
                        .help("command to run")
                        .min_values(1),
                ),
        )
        .subcommand(
            SubCommand::with_name("firefox")
                .about("Open web page remotely")
                .arg(
                    Arg::with_name("url")
                        .required(true)
                        .index(1)
                        .help("url to visit"),
                ),
        )
        .subcommand(
            SubCommand::with_name("send")
                .about("Loads file remotely")
                .arg(
                    Arg::with_name("file")
                        .required(true)
                        .index(1)
                        .help("file to send"),
                ),
        )
        .subcommand(
            SubCommand::with_name("recv")
                .about("Downloads file remotely")
                .arg(
                    Arg::with_name("file")
                        .required(true)
                        .index(1)
                        .help("file to receive"),
                ),
        )
        .subcommand(
            SubCommand::with_name("like")
                .about("Put likes on forum.olinfo.it")
                .arg(
                    Arg::with_name("user")
                        .required(true)
                        .index(1)
                        .help("user to put likes"),
                ),
        )
        .get_matches();

    let targets: Vec<(u8, u8)> = parse_targets(matches.value_of("target").unwrap()).unwrap();

    let mut handles = Vec::new();
    if let Some(matches) = matches.subcommand_matches("run") {
        let command: String = matches
            .values_of("command")
            .unwrap()
            .collect::<Vec<&str>>()
            .join(" ");
        for (row, column) in targets {
            let c = command.clone();
            handles.push((
                row,
                column,
                Worker::new(move || OlinfoClient::new(row, column)?.run(c)),
            ));
        }
    } else if let Some(matches) = matches.subcommand_matches("firefox") {
        let command = format!(
            "DISPLAY=\":0\" firefox \"{}\"",
            matches.value_of("url").unwrap()
        );
        for (row, column) in targets {
            let c = command.clone();
            handles.push((
                row,
                column,
                Worker::new(move || OlinfoClient::new(row, column)?.run(c)),
            ));
        }
    } else if let Some(matches) = matches.subcommand_matches("send") {
        let file = matches.value_of("file").unwrap().to_string();
        for (row, column) in targets {
            let f = file.clone();
            handles.push((
                row,
                column,
                Worker::new(move || {
                    OlinfoClient::new(row, column)?.send(f)?;
                    Ok((String::new(), String::new()))
                }),
            ));
        }
    } else if let Some(matches) = matches.subcommand_matches("recv") {
        let file = matches.value_of("file").unwrap().to_string();
        for (row, column) in targets {
            let f = file.clone();
            handles.push((
                row,
                column,
                Worker::new(move || {
                    OlinfoClient::new(row, column)?.recv(f)?;
                    Ok((String::new(), String::new()))
                }),
            ));
        }
    } else if let Some(matches) = matches.subcommand_matches("like") {
        let user = matches.value_of("user").unwrap().to_string();
        for (row, column) in targets {
            let u = user.clone();
            handles.push((
                row,
                column,
                Worker::new(move || OlinfoClient::new(row, column)?.like(u)),
            ));
        }
    }

    let clear = || {
        if cfg!(target_os = "windows") {
            Command::new("cls").status().unwrap();
        } else {
            Command::new("clear").status().unwrap();
        }
    };

    let term = crossterm::Crossterm::new();

    clear();
    term.cursor().hide().unwrap();

    let mut state = 0;
    let spinner = "\u{25d0}\u{25d3}\u{25d1}\u{25d2}".to_string();
    loop {
        term.cursor().goto(0, 0).unwrap();
        let mut running = 0;
        for (r, c, w) in &mut handles {
            term.terminal()
                .clear(crossterm::ClearType::CurrentLine)
                .unwrap();
            match w.get_status() {
                WorkerStatus::Running => {
                    running += 1;
                    term.terminal()
                        .write(format!(
                            "{}{}{} running {}\n",
                            crossterm::SetFg(crossterm::Color::DarkYellow),
                            (64 + *r) as char,
                            c,
                            spinner
                                .chars()
                                .nth((state + (*r + *c) as usize) % 4)
                                .unwrap(),
                        ))
                        .unwrap()
                }
                WorkerStatus::Resolved => term
                    .terminal()
                    .write(format!(
                        "{}{}{} terminated successfully\n",
                        crossterm::SetFg(crossterm::Color::Green),
                        (64 + *r) as char,
                        c,
                    ))
                    .unwrap(),
                WorkerStatus::Rejected => term
                    .terminal()
                    .write(format!(
                        "{}{}{} terminated with errors\n",
                        crossterm::SetFg(crossterm::Color::Red),
                        (64 + *r) as char,
                        c,
                    ))
                    .unwrap(),
            };
        }
        if running == 0 {
            break;
        }
        thread::sleep(Duration::from_millis(250));
        state = (state + 1) % 4;
    }

    clear();
    term.cursor().goto(0, 0).unwrap();
    term.cursor().show().unwrap();
    for (r, c, t) in handles {
        match t.join() {
            Ok((o, e)) => {
                term.terminal()
                    .write(format!(
                        "{}{}{} terminated successfully\n",
                        crossterm::SetFg(crossterm::Color::Green),
                        (64 + r) as char,
                        c,
                    ))
                    .unwrap();
                let stdout = o.trim_end().to_string();
                let stderr = e.trim_end().to_string();
                if stdout.len() > 0 {
                    term.terminal()
                        .write(format!(
                            "{}{}\n",
                            crossterm::SetFg(crossterm::Color::Reset),
                            stdout,
                        ))
                        .unwrap();
                }
                if stderr.len() > 0 {
                    term.terminal()
                        .write(format!(
                            "{}{}\n",
                            crossterm::SetFg(crossterm::Color::Yellow),
                            stderr,
                        ))
                        .unwrap();
                }
            }
            Err(e) => {
                term.terminal()
                    .write(format!(
                        "{}{}{} terminated with errors: {}{}\n",
                        crossterm::SetFg(crossterm::Color::Red),
                        (64 + r) as char,
                        c,
                        crossterm::SetFg(crossterm::Color::Reset),
                        e,
                    ))
                    .unwrap();
            }
        }
    }
}
