extern crate rustfest;

use std::io::{self, BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;

use rustfest::ident::Query;

fn handle_connection(reader: &mut BufReader<TcpStream>) -> io::Result<()> {
    let ip = try!(reader.get_ref().peer_addr()).ip();
    let mut request = String::new();
    while try!(reader.read_line(&mut request)) > 0 {
        let query: Query = request.parse().unwrap();
        println!("got query \"{}\" from {}", query, ip);
        let reply = query.process(&ip);
        try!(reader.get_mut().write_all(reply.to_string().as_ref()));

        request.clear();
    }

    println!("got EOF, ending thread");
    Ok(())
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:113").unwrap();
    println!("listening started, ready to accept");
    for stream in listener.incoming() {
        let stream = stream.unwrap();
        thread::spawn(move || {
            // connection succeeded
            let mut reader = BufReader::new(stream);
            handle_connection(&mut reader)
        });
    }
}
