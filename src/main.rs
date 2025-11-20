mod errors;
mod icmp;
mod ping;

use std::{backtrace, env};
use std::fs::OpenOptions;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::path::PathBuf;
use std::time::Duration;

use clap::Parser;
use colorize::AnsiColor;
use rayon::iter::{ ParallelIterator, IntoParallelIterator };
use serde::Deserialize;

use crate::ping::ping;

#[derive(Deserialize)]
struct ServerConfig {
    pub name: String,
    pub ip: Ipv4Addr
}

#[derive(Deserialize)]
struct Config {
    pub servers: Vec<ServerConfig>
}

fn find_config() -> Result<PathBuf, String> {
    let home = env::var("HOME");
    match home {
        Ok(home) => {
            let config_path = PathBuf::new()
                .join(home)
                .join(".mcenroe.yaml");
            Ok(config_path)
        }
        Err(_) => {
            Err(format!("Failed to find config file."))
        }
    }
}

#[derive(Clone)]
struct PingResult {
    pub name: String,
    pub ip: Ipv4Addr,
    pub success: bool,
    pub message: String,
}

fn do_ping(server: ServerConfig) -> PingResult {
    let ServerConfig{ name, ip, .. } = server;
    match ping(ip, Duration::from_millis(300)) {
        Ok(_) => {
            PingResult { name, ip, success: true, message: format!("") }
        }
        Err(e) => {
            PingResult { name, ip, success: false, message: format!("Connection failed: {e:?}") }
        }
    }
}

enum Colour {
    Bad,
    Good,
    Neutral
}

impl Colour {
    pub fn wrap(&self, s: String, mode: bool) -> String {
        match mode {
            true => {
                match self {
                    Self::Bad => format!("%F{{red}}{s}%f"),
                    Self::Good => format!("%F{{green}}{s}%f"),
                    Self::Neutral => format!("%F{{blue}}{s}%f"),
                }
            },
            false => {
                match self {
                    Self::Bad => s.red(),
                    Self::Good => s.green(),
                    Self::Neutral => s.blue(),
                }
            }
        }
    }
}

#[derive(Parser)]
struct Args {
    #[arg(short, long)]
    zsh: bool,

    #[arg(short, long)]
    short: bool,
    
    #[arg(short, long)]
    debug: bool,
}

fn main() {
    let args = Args::parse();

    let config_path = match find_config() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("{e}");
            return;
        }
    };
    let f = OpenOptions::new().read(true).open(config_path).unwrap();
    let config: Config = serde_yaml::from_reader(f).unwrap();
    
    let res: Vec<PingResult> = config.servers.into_par_iter().map(do_ping).collect();

    let mut fails = 0usize;
    let mut messages = Vec::new();
    for res in &res {
        if res.success {
            messages.push(Colour::Good.wrap(res.name.clone(), args.zsh));
        }
        else {
            messages.push(Colour::Bad.wrap(res.name.clone(), args.zsh));
            fails += 1;
        }
    }

    let overall_colour = match fails {
        0 => {
            Colour::Good
        },
        _ => {
            Colour::Bad
        }
    };

    if args.short {
        let bad_conns: Vec<_> = res
            .into_iter()
            .filter(|res|!res.success)
            .map(|res| Colour::Bad.wrap(res.name, args.zsh))
            .collect();
        let mut bad_conns = bad_conns.join(&Colour::Neutral.wrap("|".into(), args.zsh));
        if bad_conns.len() > 0 {
            bad_conns = format!("{}{}", Colour::Neutral.wrap("|".into(), args.zsh), bad_conns);
        }

        print!(
            "{}{}{}{}",
            Colour::Neutral.wrap("[".into(), args.zsh),
            overall_colour.wrap(format!("{}/{}", messages.len() - fails, messages.len()), args.zsh),
            bad_conns,
            Colour::Neutral.wrap("]".into(), args.zsh),
        );
    }
    else if args.debug && !args.zsh {
        for res in res {
            if res.success {
                println!("Ping {} ({}) {}", res.name, res.ip, "ok".green());
            }
            else {
                println!("Ping {} ({}) {}: {}", res.name, res.ip, "failed".red(), res.message);
            }
        }
    }
    else {
        print!(
            "{}{}{}",
            Colour::Neutral.wrap("[".into(), args.zsh),
            messages.join(&Colour::Neutral.wrap("|".into(), args.zsh)),
            Colour::Neutral.wrap("]".into(), args.zsh),
        );
    }

    if !args.zsh && !args.debug {
        println!("");
    }


}
