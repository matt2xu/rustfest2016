extern crate futures;
extern crate tokio_core;
extern crate tokio_proto;
extern crate tokio_service;

extern crate rustfest;

use std::io;

use futures::stream::Empty;

use tokio_core::reactor::Core;
use tokio_core::net::{TcpStream};
use tokio_proto::{pipeline, server};
use tokio_service::simple_service;

use rustfest::{ident, transport};

fn main() {
    let mut core = Core::new().unwrap();
    let handle = core.handle();

    // Create the new TCP listener
    let addr = "0.0.0.0:113".parse().unwrap();

    server::listen(&handle, addr, move |stream: TcpStream| {
        let ip = stream.peer_addr().unwrap().ip();
        println!("incoming connection from {}", ip);

        let service = simple_service(move |request: String| {
            let query: ident::Query = request.parse().unwrap();
            println!("got query \"{}\" from {}", query, ip);
            let reply = query.process(&ip);

            // Return the response as an immediate future
            futures::finished(pipeline::Message::WithoutBody(reply.to_string()) as pipeline::Message<String, Empty<(), io::Error>>)
        });
        pipeline::Server::new(service, transport::new_line_transport(stream))
    }).unwrap();

    println!("Listening on {}", addr);
    core.run(futures::empty::<(), ()>()).unwrap();
}
