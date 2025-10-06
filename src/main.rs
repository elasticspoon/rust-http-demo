use std::fmt::{self, Display};
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4, TcpStream};
use std::{env, net::TcpListener, process};

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
                handle_connection(stream);
            }
            Err(err) => println!("failled to accept connection: {:?}", err),
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
    OK,
}

impl Display for HttpCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (code, val) = match self {
            HttpCode::OK => (200, "OK".to_string()),
        };
        write!(f, "{} {}", code, val)
    }
}

fn handle_connection(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&stream);

    let http_request: Vec<_> = buf_reader
        .lines()
        .map(|res| res.unwrap())
        .take_while(|line| !line.is_empty())
        .collect();

    println!("Request: {http_request:#?}");

    let body = fs::read_to_string("index.html").unwrap();

    let response = HttpResponse {
        code: HttpCode::OK,
        body,
    };

    stream.write_all(response.to_string().as_bytes()).unwrap();
}
