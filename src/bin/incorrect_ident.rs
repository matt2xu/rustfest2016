extern crate env_logger;
extern crate futures;
extern crate tokio_proto;
#[macro_use]
extern crate tokio_core;

extern crate rustfest;

use std::io::{self, BufRead, BufReader, Write};
use std::net::IpAddr;

use futures::{Future, Poll, Async};
use futures::stream::Stream;
use tokio_core::net::{TcpListener, TcpStream};
use tokio_core::reactor::Core;

use rustfest::ident;

struct IdentHandler {
    reader: BufReader<TcpStream>,
    ip: IpAddr,
    request: String
}

impl IdentHandler {
    fn new(stream: TcpStream) -> IdentHandler {
        let ip = stream.peer_addr().unwrap().ip();
        println!("got connection from {}", ip);

        IdentHandler {
            reader: BufReader::new(stream),
            ip: ip,
            request: String::new()
        }
    }

    fn handle(&mut self) -> String {
        let reply = {
            let line = &self.request;
            println!("got request: \"{}\" from {}", line, self.ip);

            let query: ident::Query = line.parse().unwrap();
            query.process(&self.ip)
        };

        self.request.clear();

        reply.to_string()
    }
}

impl Future for IdentHandler {
    type Item = ();
    type Error = io::Error;

    fn poll(&mut self) -> Poll<(), io::Error> {
        if try!(self.reader.read_line(&mut self.request)) > 0 {
            let reply = self.handle();
            try!(self.reader.get_ref().write_all(reply.as_ref()));

            self.request.clear();
            return Ok(Async::NotReady);
        }

        // EOF
        Ok(Async::Ready(()))
    }
}

fn main() {
    env_logger::init().unwrap();

    let mut core = Core::new().unwrap();
    let handle = core.handle();

    // Create the new TCP listener
    let addr = "0.0.0.0:113".parse().unwrap();
    let listener = TcpListener::bind(&addr, &handle).unwrap();

    let srv = listener.incoming().for_each(|(stream, _addr)| {
        // Do something with the stream
        let handler = IdentHandler::new(stream);
        handle.spawn(handler.then(|_| Ok(())));
        Ok(())
    });

    println!("listening on {:?}", addr);

    core.run(srv).unwrap();
}
