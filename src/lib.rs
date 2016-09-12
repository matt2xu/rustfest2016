extern crate bytes;
// The `tokio_core` crate contains the async IO runtime.
extern crate tokio_core as tokio;
// The `tokio_proto` crate contains the abstractions and building blocks for
// quickly implementing a protocol client or server.
extern crate tokio_proto as proto;

pub mod ident;
pub mod transport;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}
