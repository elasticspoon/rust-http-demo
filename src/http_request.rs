use std::collections::HashMap;
use std::fmt;
use std::fmt::Display;
use std::io::BufRead;
use std::str::FromStr;

#[derive(Debug)]
pub struct HttpRequest {
    pub verb: HttpVerb,
    pub protocol: HttpProtocol,
    pub path: String,
    pub headers: HashMap<String, String>,
    pub body: Option<String>,
}

impl HttpRequest {
    pub fn build(request: &mut dyn BufRead) -> Result<HttpRequest, ()> {
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
            let body_len: usize = len.parse().map_err(|err| {
                eprintln!("ERROR: invalid Content-Length '{len}': {err}");
            })?;
            let mut buffer = vec![0u8; body_len];
            // TODO: if the content length is longer than provided bytes this
            // will just hang. we want some sort of a timeout somewhere
            request.read_exact(&mut buffer).map_err(|err| {
                eprintln!("ERROR: expected to read {body_len} bytes from body: {err}");
            })?;
            Some(String::from_utf8(buffer).map_err(|err| {
                eprintln!("ERROR: converting body to UTF8: {err}");
            })?)
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
impl Display for HttpRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let body = self.body.as_deref().unwrap_or("");
        let headers = self
            .headers
            .iter()
            .map(|(key, value)| format!("{key}: {value}"))
            .collect::<Vec<String>>()
            .join("\n");
        write!(
            f,
            "{verb} {path} {protocol}\r\n{headers}\r\n\r\n{body:?}",
            verb = self.verb,
            path = self.path,
            protocol = self.protocol,
            headers = headers,
        )
    }
}

fn build_start_line(start_line: String) -> Result<(HttpVerb, String, HttpProtocol), ()> {
    let mut parts = start_line.split(" ");
    if let (Some(verb), Some(path), Some(protocol)) = (parts.next(), parts.next(), parts.next()) {
        let path = path.to_string();
        let verb = HttpVerb::from_str(verb)?;
        let protocol = HttpProtocol::from_str(protocol)?;
        Ok((verb, path, protocol))
    } else {
        eprintln!("invalid start_line");
        Err(())
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

pub struct HttpResponse {
    pub code: HttpCode,
    pub body: String,
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

pub enum HttpCode {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        io::{BufReader, Cursor, Write},
        net::{Ipv4Addr, SocketAddr, SocketAddrV4, TcpStream},
        time::Duration,
    };

    fn write_to_port(port: u16, content: String) -> String {
        let addr = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), port));
        let mut stream = TcpStream::connect_timeout(&addr, Duration::from_secs(2))
            .expect("should have bound to port 3000");
        stream
            .write_all(content.as_bytes())
            .expect("should have written to socket");

        let buf_reader = BufReader::new(&stream);
        buf_reader
            .lines()
            .map(|result| result.unwrap())
            .take_while(|line| !line.is_empty())
            .collect::<Vec<String>>()
            .join("\n")
    }

    #[test]
    fn test_build_success() {
        let raw_request =
            b"GET /api/users HTTP/1.1\r\nHost: example.com\r\nUser-Agent: test\r\n\r\n";
        let mut cursor = Cursor::new(raw_request);
        let result = HttpRequest::build(&mut cursor);

        assert!(result.is_ok());
        let request = result.unwrap();
        assert_eq!(request.verb, HttpVerb::Get);
        assert_eq!(request.path, "/api/users");
        assert_eq!(request.protocol, HttpProtocol::OnePointOne);
        assert_eq!(
            *request.headers.get("Host").unwrap(),
            "example.com".to_string()
        );
        assert_eq!(
            *request.headers.get("User-Agent").unwrap(),
            "test".to_string()
        );
        assert_eq!(request.body, None);
    }

    #[test]
    fn test_build_success_with_body() {
        let raw_request =
            b"GET /api/users HTTP/1.1\r\nHost: example.com\r\nContent-Length: 5\r\n\r\nabcde";
        let mut cursor = Cursor::new(raw_request);
        let result = HttpRequest::build(&mut cursor);

        assert!(result.is_ok());
        let request = result.unwrap();
        assert_eq!(request.verb, HttpVerb::Get);
        assert_eq!(request.path, "/api/users");
        assert_eq!(request.protocol, HttpProtocol::OnePointOne);
        assert_eq!(
            *request.headers.get("Host").unwrap(),
            "example.com".to_string()
        );
        assert_eq!(
            *request.headers.get("Content-Length").unwrap(),
            "5".to_string()
        );
        assert_eq!(request.body.unwrap(), "abcde".to_string());
    }

    #[test]
    fn test_build_failure_invalid_verb() {
        let raw_request =
            b"BAD /api/users HTTP/1.1\r\nHost: example.com\r\nUser-Agent: test\r\n\r\n";
        let mut cursor = Cursor::new(raw_request);

        let result = HttpRequest::build(&mut cursor);

        assert!(result.is_err());
    }

    #[test]
    fn test_build_failure_invalid_protocol() {
        let raw_request =
            b"GET /api/users HTTP/3.9\r\nHost: example.com\r\nUser-Agent: test\r\n\r\n";
        let mut cursor = Cursor::new(raw_request);

        let result = HttpRequest::build(&mut cursor);

        assert!(result.is_err());
    }

    #[test]
    fn test_build_failure_invalid_content_length() {
        let raw_request =
            b"GET /api/users HTTP/1.1\r\nHost: example.com\r\nContent-Length: hello\r\n\r\n";
        let mut cursor = Cursor::new(raw_request);

        let result = HttpRequest::build(&mut cursor);

        assert!(result.is_err());
    }

    #[test]
    fn test_build_failure_invalid_content() {
        let raw_request =
            b"GET /api/users HTTP/1.1\r\nHost: example.com\r\nContent-Length: 5\r\n\r\n";
        let mut cursor = Cursor::new(raw_request);

        let result = HttpRequest::build(&mut cursor);

        assert!(result.is_err());
    }
}
