use constants;

pub type Magic = u32;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Network {
    Mainnet,
    Polaris,
    Unitest,
    Other(u32),
}

impl Network {
    pub fn magic(&self) -> Magic {
        match *self {
            Network::Mainnet => constants::NETWORK_MAGIC_MAINNET,
            Network::Polaris => constants::NETWORK_MAGIC_POLARIS,
            Network::Unitest => constants::NETWORK_MAGIC_UNITEST,
            Network::Other(value) => value,
        }
    }

    pub fn port(&self) -> u16 {
        match *self {
            Network::Mainnet | Network::Other(_) => 20338,
            Network::Polaris => 20338,
            Network::Unitest => 20338,
        }
    }

    pub fn rpc_port(&self) -> u16 {
        match *self {
            Network::Mainnet | Network::Other(_) => 20334,
            Network::Polaris => 20334,
            Network::Unitest => 20334,
        }
    }
}
