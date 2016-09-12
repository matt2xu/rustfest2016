use std::fmt;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::net::IpAddr;
use std::str::FromStr;

pub struct Query {
    pub server: u16,
    pub client: u16
}

impl fmt::Display for Query {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}, {}", self.server, self.client)
    }
}

fn get_port(port: &str) -> u16 {
    u16::from_str_radix(port, 16).unwrap()
}

impl Query {
    fn read_from_proc(&self, filename: &str, ip_hex: &str) -> io::Result<RespType> {
        let file = try!(File::open(filename));
        let reader = BufReader::new(file);
        let mut lines = reader.lines();

        // skip first line
        try!(lines.next().unwrap());

        for line in lines {
            let line = try!(line);
            let mut fields = line.split_whitespace();

            fields.next().unwrap();
            let local = fields.next().unwrap().split(':');
            let local_port = get_port(local.last().unwrap());

            let mut remote = fields.next().unwrap().split(':');
            let remote_addr = remote.next().unwrap();
            let remote_port = get_port(remote.next().unwrap());
            if ip_hex == remote_addr && self.server == local_port && self.client == remote_port {
                let uid = fields.nth(4).unwrap();
                println!("found connection used by uid {}", uid);
                let user = uid.to_string();
                return Ok(RespType::IdentReply{
                    os: "UNIX".to_string(),
                    charset: None,
                    user_id: user
                });
            }
        }
        Ok(RespType::ErrorReply(IdentError::NoUser))
    }

    pub fn process(&self, ip: &IpAddr) -> Reply {
        let (filename, ip_hex): (&str, String) = match *ip {
            IpAddr::V4(ipv4) => {
                ("/proc/net/tcp", ipv4.octets().iter().rev().map(|b| format!("{:02X}", b)).collect())
            }
            IpAddr::V6(ipv6) => {
                ("/proc/net/tcp6", ipv6.segments().iter().map(|b| format!("{:02X}", b)).collect())
            }
        };

        println!("remote addr: {}", ip_hex);

        let result =
            if self.server == 0 || self.client == 0 {
                Ok(RespType::ErrorReply(IdentError::InvalidPort))
            } else {
                self.read_from_proc(filename, &ip_hex)
            };
        Reply {
            server: self.server,
            client: self.client,
            response: 
                match result {
                    Ok(response) => response,
                    Err(error) => RespType::ErrorReply(IdentError::Ext(error.to_string()))
                }
        }
    }
}

pub struct Reply {
    server: u16,
    client: u16,
    response: RespType
}

pub enum RespType {
    IdentReply {
        os: String,
        charset: Option<String>,
        user_id: String
    },
    ErrorReply(IdentError)
}

pub enum IdentError {
    InvalidPort,
    NoUser,
    HiddenUser,
    UnknownError,
    Ext(String)
}

impl FromStr for Query {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // parse ports, replace invalid port numbers by 0
        let mut ports = s.split(',').map(|s| s.trim().parse::<u16>().unwrap_or(0));
        let server = ports.next().unwrap_or(0);
        Ok(Query {
            server: server,
            client: ports.next().unwrap_or(0)
        })
    }
}

impl fmt::Display for Reply {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(write!(f, "{}, {}: ", self.server, self.client));
        match self.response {
            RespType::IdentReply {ref os, ref charset, ref user_id} => {
                try!(write!(f, "USERID: {}", os));
                if let Some(ref charset) = *charset {
                    try!(write!(f, ", {}", charset));
                }
                write!(f, ": {}\r\n", user_id)
            }
            RespType::ErrorReply(ref err) => write!(f, "ERROR: {}\r\n", err)
        }
    }
}

impl fmt::Display for IdentError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            IdentError::InvalidPort => write!(f, "INVALID-PORT"),
            IdentError::NoUser => write!(f, "NO-USER"),
            IdentError::HiddenUser => write!(f, "HIDDEN-USER"),
            IdentError::UnknownError => write!(f, "UNKNOWN-ERROR"),
            IdentError::Ext(ref ext) => write!(f, "{}", ext)
        }
    }
}
