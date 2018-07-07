use byteorder::{ByteOrder, LittleEndian};
use bytes::{BufMut, BytesMut};
use serde_json as json;
use std::io;
use std::net;
pub use tokio_io::codec::{Decoder, Encoder};

use super::crypto;
use super::primitives::hash::H32;

pub mod network;

#[derive(Serialize, Deserialize, Debug)]
pub struct Version {
    pub version: u32,
    pub services: u64,
    pub timestamp: u32,
    pub sync_port: u16,
    pub info_port: u16,
    pub cons_port: u16,
    pub cap: [u8; 32],
    pub nonce: u64,
    pub useragent: u8,
    pub start_height: u64,
    pub relay: u8,
    pub consensus: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct VersionAck {
    pub consensus: bool,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct Ping {
    pub height: u32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Pong {
    pub height: u32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Transaction {
    //todo
    pub txn: u32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct NodeInfo {
    pub time: i64,
    pub services: u64,
    pub ip: net::IpAddr,
    pub port: u16,
    pub cons_port: u16,
    pub id: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct NodesInfo {
    pub nodes: Vec<NodeInfo>,
}

pub enum Message {
    Version(Version),
    VersionAck(VersionAck),
    Ping(Ping),
    Pong(Pong),
    Transaction(Transaction),
    GetNodes,
    NodesInfo(NodesInfo),
}

const MSG_CMD_LEN: usize = 12;
const MSG_CMD_OFFSET: usize = 4;
const MSG_LEN_OFFSET: usize = 4 + MSG_CMD_LEN;
const MSG_CHECKSUM_OFFSET: usize = MSG_HEADER_LEN - MSG_CHECKSUM_LEN;
const MSG_CHECKSUM_LEN: usize = 4;
const MSG_HEADER_LEN: usize = 24;

//    const MSG_MAX_SIZE :usize = 20*1024*1024;

struct MsgHeader {
    magic: network::Magic,
    cmd: [u8; MSG_CMD_LEN],
    length: usize, // save as u32
    checksum: H32,
}

impl MsgHeader {
    fn parse(hdr: &[u8]) -> MsgHeader {
        debug_assert!(hdr.len() == MSG_HEADER_LEN);

        let magic = LittleEndian::read_u32(hdr);
        let mut cmd = [0; MSG_CMD_LEN];
        cmd.copy_from_slice(&hdr[MSG_CMD_OFFSET..MSG_LEN_OFFSET + MSG_CMD_LEN]);
        let length = LittleEndian::read_u32(&hdr[MSG_LEN_OFFSET..MSG_LEN_OFFSET + 4]) as usize;
        let checksum =
            H32::from_slice(&hdr[MSG_CHECKSUM_OFFSET..MSG_CHECKSUM_OFFSET + MSG_CHECKSUM_LEN]);

        MsgHeader {
            magic,
            cmd,
            length,
            checksum,
        }
    }
}

pub struct MessageCodec {
    network: network::Network,
}

impl MessageCodec {
    pub fn new(network: network::Network) -> Self {
        Self { network }
    }
}

impl Decoder for MessageCodec {
    type Item = Message;
    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if src.len() < MSG_HEADER_LEN {
            return Ok(None);
        }

        let hdr = MsgHeader::parse(&src[..MSG_HEADER_LEN]);

        if src.len() < hdr.length + MSG_HEADER_LEN {
            return Ok(None);
        }

        let payload = src.split_to(hdr.length + MSG_HEADER_LEN);
        let checksum = crypto::checksum(&payload);
        if hdr.magic != self.network.magic() || checksum != hdr.checksum {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "wrong checksum"));
        }

        match &hdr.cmd {
            b"version\0\0\0\0\0" => Ok(Some(Message::Version(json::from_slice(&payload)?))),
            b"verack\0\0\0\0\0\0" => Ok(Some(Message::VersionAck(json::from_slice(&payload)?))),
            b"ping\0\0\0\0\0\0\0\0" => Ok(Some(Message::Ping(json::from_slice(&payload)?))),
            b"pong\0\0\0\0\0\0\0\0" => Ok(Some(Message::Pong(json::from_slice(&payload)?))),
            b"getaddr\0\0\0\0\0" => Ok(Some(Message::GetNodes)),
            b"addr\0\0\0\0\0\0\0\0" => Ok(Some(Message::NodesInfo(json::from_slice(&payload)?))),
            b"tx\0\0\0\0\0\0\0\0\0\0" => {
                Ok(Some(Message::Transaction(json::from_slice(&payload)?)))
            }
            _ => Ok(None),
        }
    }
}

impl Encoder for MessageCodec {
    type Item = Message;
    type Error = io::Error;

    fn encode(&mut self, item: Self::Item, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let (cmd, payload) = match item {
            Message::Version(payload) => {
                let msg = json::to_string(&payload).unwrap();
                (b"version\0\0\0\0\0", msg)
            }
            Message::VersionAck(payload) => {
                let msg = json::to_string(&payload).unwrap();
                (b"verack\0\0\0\0\0\0", msg)
            }
            Message::Ping(payload) => {
                let msg = json::to_string(&payload).unwrap();
                (b"ping\0\0\0\0\0\0\0\0", msg)
            }
            Message::Pong(payload) => {
                let msg = json::to_string(&payload).unwrap();
                (b"pong\0\0\0\0\0\0\0\0", msg)
            }
            Message::GetNodes => (b"getaddr\0\0\0\0\0", "".into()),
            Message::NodesInfo(payload) => {
                let msg = json::to_string(&payload).unwrap();
                (b"addr\0\0\0\0\0\0\0\0", msg)
            }
            Message::Transaction(payload) => {
                let msg = json::to_string(&payload).unwrap();
                (b"tx\0\0\0\0\0\0\0\0\0\0", msg)
            }
        };

        let payload: &[u8] = payload.as_ref();

        let length = payload.len();
        let checksum = crypto::checksum(payload);

        dst.reserve(length + MSG_HEADER_LEN);
        dst.put_u32_le(self.network.magic());
        dst.put_slice(&cmd[..]);
        dst.put_u32_le(length as u32);
        dst.put_slice(&checksum);
        dst.put(payload);

        Ok(())
    }
}
