use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::process::Command;

#[derive(Debug)]
struct Config {
    batteries: Vec<String>,
    delay_seconds: u64,
    warning: u8,
    critical: u8,
    danger: u8,
    dangercmd: String,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            batteries: vec![String::from("BAT0"), String::from("BAT1")],
            delay_seconds: 60,
            warning: 25,
            critical: 10,
            danger: 3,
            dangercmd: String::from("notify-send -u critical 'Critical' 'Hibernated system due to low battery' && systemctl hibernate"),
        }
    }
}

fn main() {
    let usage_string: String = format!(
        "\
USAGE:
    batt [OPTIONS]

OPTIONS:
    {:width$}Launch batt as a daemon
    {:width$}Show this help message",
        "-d, --daemonize",
        "-h, --help",
        width = 20
    );

    let mut daemonize = false;

    let mut args = std::env::args();
    args.next();
    for arg in args {
        match arg.as_str() {
            "-d" | "--daemonize" => daemonize = true,
            "-h" | "--help" => {
                println!("A simple battery monitor\n{}", usage_string);
                return;
            }
            _ => {
                println!("Error: unrecognized option '{}'\n\n{}", arg, usage_string);
                std::process::exit(1);
            }
        };
    }

    if daemonize {
        unsafe {
            libc::daemon(1, 1);
        }
    }

    let config = Config::default();
    
    let mut last_max_percentage = 101;
    let battery_paths: Vec<(PathBuf, PathBuf)> = config
        .batteries
        .iter()
        .map(|battery| {
            let mut path = PathBuf::from("/sys/class/power_supply/");
            path.push(battery);
            let mut status_path = path.clone();
            status_path.push("status");
            let mut percentage_path = path;
            percentage_path.push("capacity");
            (status_path, percentage_path)
        })
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
