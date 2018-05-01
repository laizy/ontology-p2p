extern crate byteorder;
extern crate bytes;
extern crate tokio_io;
#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;


pub mod p2p {
    pub mod utils {
        pub fn checksum(data: &[u8]) -> [u8;4] {
            return [0,0,0,0]
        }
    }

    use byteorder::{ByteOrder, LittleEndian};
    use bytes::{BufMut, BytesMut};
    use std::io;
    use tokio_io::codec::{Decoder, Encoder};
    use serde_json as json;

    #[derive(Serialize, Deserialize, Debug)]
    pub struct Version {
        version: u32,
        services: u64,
        timestamp: u32,
        sync_port: u16,
        info_port: u16,
        cons_port: u16,
        cap: [u8; 32],
        nonce: u64,
        useragent: u8,
        start_height: u64,
        relay: u8,
        consensus: bool,
    }

    pub enum Message {
        Version(Version),
    }

    const MSG_CMD_LEN: usize = 12;
    const MSG_CMD_OFFSET: usize = 4;
    const MSG_LEN_OFFSET: usize = 4 + MSG_CMD_LEN;
    const MSG_CHECKSUM_OFFSET : usize = MSG_HEADER_LEN - MSG_CHECKSUM_LEN;
    const MSG_CHECKSUM_LEN: usize = 4;
    const MSG_HEADER_LEN: usize = 24;
    const MSG_NET_MAGIC: u32 = 0x74746E41;

//    const MSG_MAX_SIZE :usize = 20*1024*1024;

    struct MsgHeader {
        magic: u32,
        cmd: [u8; MSG_CMD_LEN],
        length: usize, // save as u32
        checksum: [u8; MSG_CHECKSUM_LEN],
    }

    impl MsgHeader {
        fn parse(hdr: &[u8]) -> MsgHeader {
            debug_assert!(hdr.len() == MSG_HEADER_LEN);

            let magic = LittleEndian::read_u32(hdr);
            let mut cmd = [0; MSG_CMD_LEN];
            cmd.copy_from_slice(&hdr[MSG_CMD_OFFSET..MSG_LEN_OFFSET+MSG_CMD_LEN]);
            let length = LittleEndian::read_u32(&hdr[MSG_LEN_OFFSET..MSG_LEN_OFFSET+ 4]) as usize;
            let mut checksum = [0; MSG_CHECKSUM_LEN];
            checksum.copy_from_slice(&hdr[MSG_CHECKSUM_OFFSET..MSG_CHECKSUM_OFFSET+MSG_CHECKSUM_LEN]);

            MsgHeader {
                magic,
                cmd,
                length,
                checksum,
            }
        }
    }

    pub struct MessageCodec;

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
            let checksum = utils::checksum(&payload);
            if hdr.magic != MSG_NET_MAGIC || checksum != hdr.checksum {
                return Err(io::Error::new(io::ErrorKind::InvalidData, "wrong checksum"));
            }

            match &hdr.cmd {
                b"version\0\0\0\0\0" => {
                    Ok(Some(Message::Version(json::from_slice::<Version>(&payload)?)))
                }
                _ => {
                    Ok(None)
                }

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
            };

            let payload:&[u8] = payload.as_ref();

            let length = payload.len();
            let checksum = utils::checksum(payload.as_ref());

            dst.reserve(length + MSG_HEADER_LEN);
            dst.put_u32_le(MSG_NET_MAGIC);
            dst.put_slice(&cmd[..]);
            dst.put_u32_le(length as u32);
            dst.put_slice(&checksum);
            dst.put(payload);

            Ok(())
        }

    }

}


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}

