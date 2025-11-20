mod errors;
mod icmp;
mod ping;

use std::env;
use std::fs::OpenOptions;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::path::PathBuf;
use std::time::Duration;

use colorize::AnsiColor;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
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

fn main() {
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
    print!("{}() -> {}", "ping".yellow(), "[".blue());
    for res in &res {
        if res.success {
            print!(" {}", res.name.clone().green());
        }
        else {
            print!(" {}", res.name.clone().red());
            fails += 1;
        }
    }

    match fails {
        0 => {},
        _ => {
            for res in res.into_iter().filter(|res| !res.success) {
                println!("Ping {} failed: {}", res.name, res.message);
            }
        }
    }

    println!("{}", " ]".blue());

}
