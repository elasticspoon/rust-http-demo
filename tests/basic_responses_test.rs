use demo_http_server::*;
use std::{
    collections::HashMap,
    io::{BufRead, BufReader, Cursor, Write},
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
fn test_get_200() {
    let request = HttpRequest {
        verb: HttpVerb::Get,
        protocol: HttpProtocol::OnePointOne,
        path: "/".to_string(),
        headers: HashMap::from([("Host".to_string(), "localhost:3000".to_string())]),
        body: None,
    };

    let resp = write_to_port(3000, request.to_string());
    assert!(resp.contains("HTTP/1.1 200 OK"));
}

#[test]
fn test_get_400() {
    let request = HttpRequest {
        verb: HttpVerb::Get,
        protocol: HttpProtocol::OnePointOne,
        path: "/".to_string(),
        headers: HashMap::from([]),
        body: None,
    };

    let want = "HTTP/1.1 400 Bad Request";
    let resp = write_to_port(3000, request.to_string());
    assert!(resp.contains(want), "{resp} did not contain {want}");
}

#[test]
fn test_build_success() {
    let raw_request = b"GET /api/users HTTP/1.1\r\nHost: example.com\r\nUser-Agent: test\r\n\r\n";
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
    let raw_request = b"BAD /api/users HTTP/1.1\r\nHost: example.com\r\nUser-Agent: test\r\n\r\n";
    let mut cursor = Cursor::new(raw_request);

    let result = HttpRequest::build(&mut cursor);

    assert!(result.is_err());
}

#[test]
fn test_build_failure_invalid_protocol() {
    let raw_request = b"GET /api/users HTTP/3.9\r\nHost: example.com\r\nUser-Agent: test\r\n\r\n";
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
    let raw_request = b"GET /api/users HTTP/1.1\r\nHost: example.com\r\nContent-Length: 5\r\n\r\n";
    let mut cursor = Cursor::new(raw_request);

    let result = HttpRequest::build(&mut cursor);

    assert!(result.is_err());
}
