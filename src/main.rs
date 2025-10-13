use std::fmt::{self, Display};
use std::fs;
use std::io::{BufReader, Write};
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4, TcpStream};
use std::str::FromStr;
use std::{env, net::TcpListener, process};
mod http_request;
use http_request::HttpRequest;

fn main() {
    // let port = env::var("PORT").map_or_else(|_| 3000, |port| port.parse::<i32>().unwrap_or(3000));
    let port = env::var("PORT")
        .ok()
        .and_then(|port| port.parse::<i32>().ok())
        .unwrap_or(3000);
    let addr = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 3000));
    let listener = TcpListener::bind(addr).unwrap_or_else(|err| {
        eprintln!("Failed to bind to localhost:{}: {}", port, err);
        process::exit(1);
    });

    println!("Listening on {}", listener.local_addr().unwrap());
    for conn in listener.incoming() {
        match conn {
            Ok(stream) => {
                println!("Accepted connection");
                let _ = handle_connection(stream);
            }
            Err(err) => println!("failled to accept connection: {:?}", err),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum HttpProtocol {
    OnePointOne,
}

impl FromStr for HttpProtocol {
    type Err = ();

    fn from_str(protocol: &str) -> Result<Self, Self::Err> {
        match protocol {
            "HTTP/1.1" => Ok(HttpProtocol::OnePointOne),
            _ => {
                eprintln!("ERROR: invalid protocol in request: '{protocol}'");
                Err(())
            }
        }
    }
}

impl Display for HttpProtocol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let protocol = match self {
            HttpProtocol::OnePointOne => "HTTP/1.1",
        };
        write!(f, "{protocol}")
    }
}
#[derive(Debug, PartialEq)]
pub enum HttpVerb {
    Get,
    Post,
    Put,
}

impl Display for HttpVerb {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let verb = match self {
            HttpVerb::Get => "GET",
            HttpVerb::Post => "POST",
            HttpVerb::Put => "PUT",
        };
        write!(f, "{verb}",)
    }
}

impl FromStr for HttpVerb {
    type Err = ();

    fn from_str(verb: &str) -> Result<Self, Self::Err> {
        match verb {
            "GET" => Ok(HttpVerb::Get),
            "POST" => Ok(HttpVerb::Post),
            "PUT" => Ok(HttpVerb::Put),
            _ => {
                eprintln!("ERROR: Invalid HTTP verb: '{verb}'");
                Err(())
            }
        }
    }
}

struct HttpResponse {
    code: HttpCode,
    body: String,
}

impl fmt::Display for HttpResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let resp_header = format!("HTTP/1.1 {}", self.code);
        write!(
            f,
            "{resp_header}\r\nContent-Length: {}\r\n\r\n{}",
            self.body.len(),
            self.body
        )
    }
}

enum HttpCode {
    Ok,
    NotFound,
}

impl Display for HttpCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (code, val) = match self {
            HttpCode::Ok => (200, "OK".to_string()),
            HttpCode::NotFound => (404, "NOT FOUND".to_string()),
        };
        write!(f, "{} {}", code, val)
    }
}

fn handle_connection(mut stream: TcpStream) -> Result<(), ()> {
    let mut buf_reader = BufReader::new(&stream);
    // TODO: how should a server respond to a malformed request?
    let http_request = HttpRequest::build(&mut buf_reader)?;
    print!("{http_request}");
    let handler = match (&http_request.verb, http_request.path.as_str()) {
        (HttpVerb::Get, "/") => Some(|_| fs::read_to_string("index.html").unwrap()),
        _ => None,
    };

    let response = match handler {
        Some(handler) => HttpResponse {
            code: HttpCode::Ok,
            body: handler(http_request),
        },
        None => HttpResponse {
            code: HttpCode::NotFound,
            body: "".to_string(),
        },
    };

    println!("\r\nResponse:\r\n{response}");
    stream.write_all(response.to_string().as_bytes()).unwrap();
    Ok(())
}
