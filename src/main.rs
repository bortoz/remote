mod remotelib;
use clap::{App, Arg, SubCommand};
use rand::Rng;
use remotelib::OlinfoClient;
use std::thread;

fn parse_target(arg: &str) -> Option<(u8, u8)> {
    let chars: Vec<char> = arg.chars().collect();
    if chars.len() != 2 || !('1' <= chars[1] && chars[1] <= '4') {
        None
    } else if 'a' <= chars[0] && chars[0] <= 'd' {
        Some((chars[0] as u8 - 96, chars[1] as u8 - 48))
    } else if 'A' <= chars[0] && chars[0] <= 'D' {
        Some((chars[0] as u8 - 64, chars[1] as u8 - 48))
    } else {
        None
    }
}

fn parse_targets(arg: &str) -> Option<Vec<(u8, u8)>> {
    if arg == "all" {
        let mut targets = Vec::new();
        for r in 1..=4 {
            for c in 1..=4 {
                targets.push((r, c));
            }
        }
        Some(targets)
    } else if arg == "rand" {
        let mut rng = rand::thread_rng();
        Some(vec![(rng.gen_range(1, 5), rng.gen_range(1, 5))])
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
                thread::spawn(move || OlinfoClient::new(row, column)?.run(c)),
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
                thread::spawn(move || OlinfoClient::new(row, column)?.run(c)),
            ));
        }
    } else if let Some(matches) = matches.subcommand_matches("send") {
        let file = matches.value_of("file").unwrap().to_string();
        for (row, column) in targets {
            let f = file.clone();
            handles.push((
                row,
                column,
                thread::spawn(move || {
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
                thread::spawn(move || {
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
                thread::spawn(move || OlinfoClient::new(row, column)?.like(u)),
            ));
        }
    }

    handles.sort_by(|(r1, c1, _), (r2, c2, _)| (r1, c1).partial_cmp(&(r2, c2)).unwrap());
    for (r, c, t) in handles {
        match t.join().unwrap() {
            Ok((o, e)) => {
                println!("{}{} terminated successfully", (64 + r) as char, c);
                let stdout = o.trim_end().to_string();
                let stderr = e.trim_end().to_string();
                if stdout.len() > 0 {
                    println!("{}", stdout);
                }
                if stderr.len() > 0 {
                    println!("{}", stderr);
                }
            }
            Err(e) => {
                println!("{}{} terminated with errors: {}", (64 + r) as char, c, e);
            }
        }
    }
}
