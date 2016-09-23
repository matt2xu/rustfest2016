extern crate env_logger;
extern crate futures;
#[macro_use]
extern crate tokio_core;

extern crate rustfest;

use std::io::{self, Read};
use std::str;

use futures::{Future, Poll, Async};
use futures::stream::Stream;
use tokio_core::io::{Io, ReadHalf, write_all};
use tokio_core::net::{TcpListener};
use tokio_core::reactor::Core;

use rustfest::ident;

struct Requests<T> {
    reader: ReadHalf<T>,
    buf: Vec<u8>,
    total_bytes: usize
}

impl<T> Requests<T> {
    fn new(reader: ReadHalf<T>) -> Requests<T> {
        Requests {
            reader: reader,
            buf: vec![0; 256],
            total_bytes: 0
        }
    }
}

impl<T: Read> Stream for Requests<T> {
    type Item = String;
    type Error = io::Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        let nbytes = try_nb!(self.reader.read(&mut self.buf[self.total_bytes .. ]));
        if nbytes == 0 {
            // EOF
            return Ok(Async::Ready(None));
        }

        self.total_bytes += nbytes;
        if let Some(index) = self.buf[.. self.total_bytes].iter().position(|b| *b == b'\r') {
            let request = str::from_utf8(&self.buf[.. index]).unwrap().to_string();
            self.total_bytes = 0;
            Ok(Async::Ready(Some(request)))
        } else {
            // incomplete request

            let len = self.buf.len();
            if self.total_bytes == len {
                // grow buffer
                self.buf.resize(len * 2, 0);
            }

            Ok(Async::NotReady)
        }
    }
}

fn main() {
    env_logger::init().unwrap();

    let mut core = Core::new().unwrap();
    let handle = core.handle();

    // Create the new TCP listener
    let addr = "0.0.0.0:113".parse().unwrap();
    let listener = TcpListener::bind(&addr, &handle).unwrap();

    let srv = listener.incoming().for_each(move |(socket, addr)| {
        println!("incoming connection from {}", addr);
        let ip = addr.ip();

        let pair = futures::lazy(|| Ok(socket.split()));
        let future = pair.and_then(move |(reader, writer)| {
            Requests::new(reader).map(move |req| {
                println!("got request \"{}\"", req);

                let query: ident::Query = req.parse().unwrap();
                query.process(&ip).to_string()
            }).fold(writer, move |writer, reply| {
                write_all(writer, reply).and_then(|(writer, _)| futures::finished(writer))
            })
        });

        handle.spawn(future.then(|_| Ok(())));
        Ok(())
    });

    println!("listening on {:?}", addr);

    core.run(srv).unwrap();
}
