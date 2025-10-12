use std::collections::HashMap;
use std::fmt::{self, Display};
use std::fs;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4, TcpStream};
use std::str::FromStr;
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

struct HttpRequest {
    verb: HttpVerb,
    protocol: HttpProtocol,
    path: String,
    headers: HashMap<String, String>,
    body: Option<String>,
}

impl HttpRequest {
    fn build(request: &mut dyn BufRead) -> Result<HttpRequest, String> {
        let header_line = request.lines().next().unwrap().unwrap();
        let (verb, path, protocol) = build_start_line(header_line)?;

        let headers: HashMap<String, String> = request
            .lines()
            .map(|res| res.unwrap())
            .take_while(|line| !line.is_empty())
            .filter_map(|line| {
                line.split_once(": ")
                    .map(|(k, v)| (k.to_string(), v.to_string()))
            })
            .collect();

        let body = if let Some(len) = headers.get("Content-Length") {
            // TODO use Box dyn Error
            let body_len: usize = len.parse().unwrap();
            let mut buffer = vec![0u8; body_len];
            request.read_exact(&mut buffer).unwrap();
            Some(String::from_utf8(buffer).unwrap())
        } else {
            None
        };

        Ok(HttpRequest {
            verb,
            path,
            protocol,
            headers,
            body,
        })
    }
}

fn build_start_line(binding: String) -> Result<(HttpVerb, String, HttpProtocol), String> {
    let mut start_line = binding.split(" ");
    if let (Some(verb), Some(path), Some(protocol)) =
        (start_line.next(), start_line.next(), start_line.next())
    {
        let path = path.to_string();
        let verb = HttpVerb::from_str(verb)?;
        let protocol = HttpProtocol::from_str(protocol)?;
        Ok((verb, path, protocol))
    } else {
        Err(format!("invalid start line: {:#?}", start_line))
    }
}

impl Display for HttpRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let body = self.body.as_deref().unwrap_or("");
        write!(
            f,
            "{verb} {path} {protocol}\r\n{headers:#?}\r\n\r\n{body:?}",
            verb = self.verb,
            path = self.path,
            protocol = self.protocol,
            headers = self.headers,
        )
    }
}

enum HttpProtocol {
    OnePointOne,
}

impl FromStr for HttpProtocol {
    type Err = String;

    fn from_str(protocol: &str) -> Result<Self, Self::Err> {
        match protocol {
            "HTTP/1.1" => Ok(HttpProtocol::OnePointOne),
            _ => Err(format!("invalid HttpProtocol: {}", protocol)),
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
enum HttpVerb {
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
    type Err = String;

    fn from_str(verb: &str) -> Result<Self, Self::Err> {
        match verb {
            "GET" => Ok(HttpVerb::Get),
            "POST" => Ok(HttpVerb::Post),
            "PUT" => Ok(HttpVerb::Put),
            _ => Err(format!("cannot covert {verb} to HttpVerb.")),
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

fn handle_connection(mut stream: TcpStream) {
    let mut buf_reader = BufReader::new(&stream);
    // let mut buf_reader = BufReader::new(&stream);
    // let mut res = String::new();
    // let req = buf_reader.read_to_string(&mut res).unwrap();
    // println!("{:#?}", res);
    // TODO: this does not do anything with the reponse body
    // it comes after \r\n\r\n and is size of Content-Length

    let http_request = HttpRequest::build(&mut buf_reader).unwrap();
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
}
