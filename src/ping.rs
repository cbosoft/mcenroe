use std::io::Read;
use std::time::{Instant, Duration};
use std::net::{Ipv4Addr, SocketAddrV4};

use socket2::{Domain, Protocol, Socket, Type};
use rand::random;

use crate::icmp::{ICMP_HEADER_SIZE, EchoRequest, EchoReply};
use crate::errors::Error;

const TOKEN_SIZE: usize = 24;
const ECHO_REQUEST_BUFFER_SIZE: usize = ICMP_HEADER_SIZE + TOKEN_SIZE;
type Token = [u8; TOKEN_SIZE];


pub fn ping(
    addr: Ipv4Addr,
    timeout: Duration,
) -> Result<(), Error> {
    let time_start = Instant::now();

    let dest = SocketAddrV4::new(addr, 0);
    let mut buffer = [0; ECHO_REQUEST_BUFFER_SIZE];

    let payload: &Token = &random();

    let request = EchoRequest {
        ident: 1,
        seq_cnt: 1,
        payload
    };

    if request.encode(&mut buffer[..]).is_err() {
        return Err(Error::InternalError.into());
    }
    let mut socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::ICMPV4))?;

    socket.set_ttl_v4(64)?;
    socket.set_write_timeout(Some(timeout))?;
    socket.send_to(&mut buffer, &dest.into())?;

    // loop until either an echo with correct ident was received or timeout is over
    loop {
        socket.set_read_timeout(Some(Duration::from_millis(100)))?;

        let mut buffer: [u8; 2048] = [0; 2048];
        let n = socket.read(&mut buffer)?;

        let reply = match EchoReply::decode(&buffer[..n]) {
            Ok(reply) => reply,
            Err(_) => continue,
        };

        let ok = {
            let mut ok = true;
            for i in 0..payload.len() {
                if payload[i] != reply.payload[i] {
                    ok = false;
                    break;
                }
            }
            ok
        };

        if ok {
            // received correct payload
            break Ok(());
        }

        if time_start.elapsed() >= timeout {
            let error = std::io::Error::new(std::io::ErrorKind::TimedOut, "Timeout occured");
            break Err(Error::IoError { error: (error) });
        }
    }
}
