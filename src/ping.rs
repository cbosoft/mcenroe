use std::io::Read;
use std::time::{Instant, Duration};
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


pub fn ping(
    addr: Ipv4Addr,
    via: Option<Vec<String>>,
    timeout: Duration,
) -> Result<(), Error> {
    let res = if let Some(via) = via {
        let mut res = ExitStatus::from_raw(255);
        for via_host in via {
            let maybe_res = Exec::cmd("ssh")
                .arg(via_host)
                .arg("ping")
                .arg("-c")
                .arg("1")
                .arg("-w")
                .arg("1")
                .arg(format!("{addr}"))
                .stdout(Redirection::Null)
                .stderr(Redirection::Null)
                .join();
            match maybe_res {
                Ok(maybe_res) => { if maybe_res.success() { res = maybe_res; break; }},
                Err(_) => (),
            }
        }
        res
    }
    else {
        Exec::cmd("ping")
            .arg("-c")
            .arg("1")
            .arg("-w")
            .arg("1")
            .arg(format!("{addr}"))
            .stdout(Redirection::Null)
            .stderr(Redirection::Null)
            .join()
            .unwrap()
    };

    if !res.success() {
        return Err(Error::InternalError);
    }
    else {
        return Ok(());
    }
}
