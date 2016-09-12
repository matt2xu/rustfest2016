// Copied from tokio-line (c) 2016 Tokio contributors
//
// Adapted for our usage:
//   - removed the write(b'\n') since our reply already contains a \r\n

use bytes::{self, Buf, BlockBuf, MutBuf};

use std::{io, str};
use std::fmt::Write;

use tokio::io::Io;
use proto::{pipeline, Parse, Serialize, Framed};

/// This defines the chunks written to our transport, i.e. the representation
/// that the `Service` deals with. In our case, the received and sent frames
/// are mostly the same (Strings with io::Error as failures), however they
/// could also be different (for example HttpRequest for In and HttpResponse
/// for Out).
pub type Frame = pipeline::Frame<String, (), io::Error>;

pub struct Parser;

impl Parse for Parser {
    type Out = Frame;

    fn parse(&mut self, buf: &mut BlockBuf) -> Option<Frame> {
        // Make sure the data is continuous in memory. BlockBuf is 'faking' a continuous buffer -
        // if you receive two TCP packets, block buf will keep two allocated memory blocks around -
        // this is very efficient for reading, but since we call the 'bytes' method below which
        // requires a single continous block of memory, we need to ask blockbuf to defrag itself. 
        if !buf.is_compact() {
            buf.compact();
        }

        // If our buffer contains a newline...
        if let Some(n) = buf.bytes().unwrap().iter().position(|b| *b == b'\n') {
            // remove this line and the newline from the buffer.
            let line = buf.shift(n);
            buf.shift(1); // Also remove the '\n'.

            // Turn this data into a UTF string and return it in a Frame.
            return match str::from_utf8(line.buf().bytes()) {
                Ok(s) => Some(pipeline::Frame::Message(s.to_string())),
                Err(_) => Some(pipeline::Frame::Error(
                        io::Error::new(io::ErrorKind::Other, "invalid string"))),
            }
        }
        None
    }
}

pub struct Serializer;

impl Serialize for Serializer {
    type In = Frame;

    fn serialize(&mut self, frame: Frame, buf: &mut BlockBuf) {
        use proto::pipeline::Frame::*;

        match frame {
            Message(text) => {
                buf.write_slice(&text.as_bytes());
            }
            Error(e) => {
                let _ = write!(bytes::Fmt(buf), "[ERROR] {}\n", e);
            }
            MessageWithBody(..) | Body(..) => {
                // Our Line protocol does not support streaming bodies
                unreachable!();
            }
            Done => {}
        }
    }
}

pub type FramedLineTransport<T> = Framed<T, Parser, Serializer>;

pub fn new_line_transport<T>(inner: T) -> FramedLineTransport<T>
    where T: Io,
{
  Framed::new(inner,
              Parser,
              Serializer,
              BlockBuf::default(),
              BlockBuf::default())
}
