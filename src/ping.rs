use std::io::Read;
use std::time::{Duration, Instant, SystemTime};
use std::net::{Ipv4Addr, SocketAddrV4};

use socket2::{Domain, Protocol, Socket, Type};
use rand::random;
use subprocess::unix::ExitStatusExt;
use subprocess::{Exec, ExitStatus, Redirection};

use crate::icmp::{ICMP_HEADER_SIZE, EchoRequest, EchoReply};
use crate::errors::Error;

const TOKEN_SIZE: usize = 24;
const ECHO_REQUEST_BUFFER_SIZE: usize = ICMP_HEADER_SIZE + TOKEN_SIZE;
type Token = [u8; TOKEN_SIZE];


fn _ping(addr: Ipv4Addr, via: Option<String>, timeout: Duration) -> bool {
    let cmd = if let Some(via) = via {
        Exec::cmd("ssh").arg("via").arg("ping")
    }
    else {
        Exec::cmd("ping")
    };
    let cmd = cmd.arg("-c")
                .arg("1")
                .arg("-w")
                .arg("1")
                .arg(format!("{addr}"))
                .stdout(Redirection::Null)
                .stderr(Redirection::Null);

    let job = cmd.start().unwrap();

    let start = SystemTime::now();
    let mut res = None;
    while (start.elapsed().unwrap() < timeout) && res.is_none() {
        res = job.poll();
    }

    match res {
        Some(res) => { res.success() },
        _ => false
    }
}


pub fn ping(
    addr: Ipv4Addr,
    via: Option<Vec<String>>,
    timeout: Duration,
) -> Result<(), Error> {
    eprintln!("pinging {addr} via {via:?} with timeout {timeout:?}");
    if let Some(via) = via {
        for via_host in via {
            if _ping(addr, Some(via_host), timeout) {
                return Ok(());
            }
        }
    }
    else {
        if _ping(addr, None, timeout) {
            return Ok(())
        }
    }

    eprintln!("{addr} is bad");
    return Err(Error::InternalError);
}
