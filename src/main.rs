mod http_request;
use demo_http_server::ThreadPool;
use http_request::HttpRequest;
use std::fmt::{self, Display};
use std::io::{BufReader, Write};
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4, TcpStream};
use std::str::FromStr;
use std::time::Duration;
use std::{env, net::TcpListener, process};
use std::{fs, thread};

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

    let pool = ThreadPool::new(4);
    println!("Listening on {}", listener.local_addr().unwrap());
    for conn in listener.incoming() {
        match conn {
            Ok(stream) => {
                println!("Accepted connection");
                pool.execute(|| {
                    let _ = handle_connection(stream);
                });
            }
            Err(err) => println!("failed to accept connection: {:?}", err),
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
    BadRequest,
}

impl Display for HttpCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (code, val) = match self {
            HttpCode::Ok => (200, "OK".to_string()),
            HttpCode::NotFound => (404, "NOT FOUND".to_string()),
            HttpCode::BadRequest => (400, "BAD REQUEST".to_string()),
        };
        write!(f, "{} {}", code, val)
    }
}

fn handle_connection(mut stream: TcpStream) -> Result<(), ()> {
    let mut buf_reader = BufReader::new(&stream);
    let http_request = HttpRequest::build(&mut buf_reader).map_err(|_| {
        let response = HttpResponse {
            code: HttpCode::BadRequest,
            body: "Bad Request".to_string(),
        };
        let _ = stream
            .write_all(response.to_string().as_bytes())
            .map_err(|err| eprintln!("Failed to write {response}: {err}"));
    })?;
    let handler = match (&http_request.verb, http_request.path.as_str()) {
        (HttpVerb::Get, "/") => |_| (HttpCode::Ok, fs::read_to_string("index.html").unwrap()),
        (HttpVerb::Get, "/sleep") => |_| {
            thread::sleep(Duration::from_secs(5));
            (HttpCode::Ok, fs::read_to_string("index.html").unwrap())
        },
        (HttpVerb::Get, "/fib") => |_| {
            fibonacci(1_000_000_000);
            (HttpCode::Ok, fs::read_to_string("index.html").unwrap())
        },
        _ => |_| {
            (
                HttpCode::NotFound,
                fs::read_to_string("not_found.html").unwrap(),
            )
        },
    };

    let (code, body) = handler(http_request);
    let response = HttpResponse { code, body };

    println!("\r\nResponse:\r\n{response}");
    let _ = stream
        .write_all(response.to_string().as_bytes())
        .map_err(|err| eprintln!("Failed to write {response}: {err}"));
    Ok(())
}

fn fibonacci(mut n: i128) {
    let mut a: i128 = 0;
    let mut b: i128 = 1;
    while n > 0 {
        match a.checked_add(b) {
            Some(sum) => {
                a = b;
                b = sum;
            }
            None => {
                a = 0;
                b = 1;
            }
        }
        n -= 1;
    }

    println!("a: {a}, b: {b}")
}
