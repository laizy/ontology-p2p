extern crate byteorder;
extern crate bytes;
extern crate tokio_io;
#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;

pub mod crypto;
pub mod primitives;

pub mod constants;
pub mod p2p;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
