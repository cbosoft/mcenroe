use std::io::Write;
use thiserror::Error;

pub const ICMP_HEADER_SIZE: usize = 8;

#[derive(Debug, Error)]
pub enum Error {
    #[error("invalid size")]
    InvalidSize,
    #[error("invalid packet")]
    InvalidPacket,
    #[error("too small header")]
    TooSmallHeader,
    #[error("invalid header size")]
    InvalidHeaderSize,
    #[error("invalid version")]
    InvalidVersion,
    #[error("unknown protocol")]
    UnknownProtocol,
}

pub struct IcmpV4;

impl IcmpV4 {
    const ECHO_REQUEST_TYPE: u8 = 8;
    const ECHO_REQUEST_CODE: u8 = 0;
    const ECHO_REPLY_TYPE: u8 = 0;
    const ECHO_REPLY_CODE: u8 = 0;
}

pub struct EchoRequest<'a> {
    pub ident: u16,
    pub seq_cnt: u16,
    pub payload: &'a [u8],
}

impl<'a> EchoRequest<'a> {
    pub fn encode(&self, buffer: &mut [u8]) -> Result<(), Error> {
        buffer[0] = IcmpV4::ECHO_REQUEST_TYPE;
        buffer[1] = IcmpV4::ECHO_REQUEST_CODE;

        buffer[4] = (self.ident >> 8) as u8;
        buffer[5] = self.ident as u8;
        buffer[6] = (self.seq_cnt >> 8) as u8;
        buffer[7] = self.seq_cnt as u8;

        if let Err(_) = (&mut buffer[8..]).write(self.payload) {
            return Err(Error::InvalidSize);
        }

        write_checksum(buffer);
        Ok(())
    }
}

pub struct EchoReply<'a> {
    pub ident: u16,
    pub seq_cnt: u16,
    pub payload: &'a [u8],
}

impl<'a> EchoReply<'a> {
    pub fn decode(buffer: &'a [u8]) -> Result<Self, Error> {
        if buffer.as_ref().len() < ICMP_HEADER_SIZE {
            return Err(Error::InvalidSize);
        }

        let type_ = buffer[0];
        let code = buffer[1];
        if type_ != IcmpV4::ECHO_REPLY_TYPE || code != IcmpV4::ECHO_REPLY_CODE {
            return Err(Error::InvalidPacket);
        }

        let ident = u16::from_be_bytes([buffer[4], buffer[5]]); // nah
        let seq_cnt = u16::from_be_bytes([buffer[6], buffer[7]]);

        let payload = &buffer[ICMP_HEADER_SIZE..];

        Ok(EchoReply {
            ident,
            seq_cnt,
            payload,
        })
    }
}

fn write_checksum(buffer: &mut [u8]) {
    let mut sum = 0u32;
    for word in buffer.chunks(2) {
        let mut part = u16::from(word[0]) << 8;
        if word.len() > 1 {
            part += u16::from(word[1]);
        }
        sum = sum.wrapping_add(u32::from(part));
    }

    while (sum >> 16) > 0 {
        sum = (sum & 0xffff) + (sum >> 16);
    }

    let sum = !sum as u16;

    buffer[2] = (sum >> 8) as u8;
    buffer[3] = (sum & 0xff) as u8;
}
