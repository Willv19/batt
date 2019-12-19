mod config;
use config::Config;

use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::process::Command;

use nix;

const USAGE_STRING: &'static str = "USAGE:
    batt [OPTIONS]

OPTIONS:
    -d, --daemonize\tLaunch batt as a daemon
    -h, --help\tShow this help message";

struct Args {
    pub daemonize: bool,
}

fn parse_args() -> Args {
    let mut daemonize = false;

    let mut args = std::env::args();
    args.next();
    for arg in args {
        match arg.as_str() {
            "-d" | "--daemonize" => daemonize = true,
            "-h" | "--help" => {
                println!("A simple battery monitor\n{}", USAGE_STRING);
                std::process::exit(0);
            }
            _ => {
                eprintln!("Error: unrecognized option '{}'\n\n{}", arg, USAGE_STRING);
                std::process::exit(1);
            }
        };
    }

    Args { daemonize }
}

fn get_status_and_percentage_path(battery: &String) -> (PathBuf, PathBuf) {
    let path = PathBuf::from("/sys/class/power_supply/".to_owned() + battery);

    let mut status_path = path.clone();
    status_path.push("status");

    let mut percentage_path = path.clone();
    percentage_path.push("capacity");

    (status_path, percentage_path)
}

fn main() {
    let Args { daemonize } = parse_args();

    if daemonize {
        match nix::unistd::daemon(true, true) {
            Ok(_) => println!("Successfully daemonized!"),
            Err(err) => {
                eprintln!("Error in daemonizing: {}", err);
                std::process::exit(1);
            }
        }
    }

    let config = Config::default();

    let mut last_max_percentage = 101;
    let battery_paths: Vec<(PathBuf, PathBuf)> = config
        .batteries
        .iter()
        .map(get_status_and_percentage_path)
        .collect();

    loop {
        let mut max_percentage = 0;

        for (status_path, percentage_path) in battery_paths.iter() {
            let mut status_file = File::open(status_path).unwrap();
            let mut status = vec![];
            status_file.read_to_end(&mut status).unwrap();
            match status.as_slice() {
                b"Unknown\n" | b"Discharging\n" => {
                    let mut percentage_file = File::open(percentage_path).unwrap();
                    let mut percentage = String::new();
                    percentage_file.read_to_string(&mut percentage).unwrap();
                    let percentage = percentage.trim_end().parse::<u8>().unwrap();
                    max_percentage = std::cmp::max(percentage, max_percentage);
                }
                b"Charging\n" | b"Full\n" | _ => {
                    max_percentage = 100;
                }
            }
        }

        if max_percentage < last_max_percentage {
            if max_percentage <= config.danger && last_max_percentage > config.danger {
                Command::new("sh")
                    .arg("-c")
                    .arg(&config.dangercmd)
                    .status()
                    .unwrap();
            } else if max_percentage <= config.critical && last_max_percentage > config.critical {
                Command::new("notify-send")
                    .arg("-u")
                    .arg("critical")
                    .arg("Battery is critically low.")
                    .status()
                    .unwrap();
            } else if max_percentage <= config.warning && last_max_percentage > config.warning {
                println!("Warning");
                Command::new("notify-send")
                    .arg("-u")
                    .arg("critical")
                    .arg("Battery is low.")
                    .status()
                    .unwrap();
            }
        }

        println!("Polling...\nLast max percentage: {}", last_max_percentage);

        last_max_percentage = max_percentage;
        std::thread::sleep(std::time::Duration::from_secs(config.delay_seconds));
    }
}
