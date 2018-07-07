extern crate bytes;
extern crate ontology_p2p as ontology;

use ontology::p2p;
use ontology::p2p::network::Network;
use ontology::p2p::Encoder;

fn main() {
    let ping = p2p::Ping { height: 1 };

    let mut buf = bytes::BytesMut::new();
    let mut codec = p2p::MessageCodec::new(Network::Mainnet);
    codec.encode(p2p::Message::Ping(ping), &mut buf).unwrap();

    println!("{:?}", buf)
}
