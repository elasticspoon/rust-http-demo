use std::collections::HashMap;
use std::fmt::Display;
use std::io::BufRead;
use std::str::FromStr;
use std::{error::Error, fmt};

use super::HttpProtocol;
use super::HttpVerb;
use crate::MalformedRequest;

#[derive(Debug)]
pub(crate) struct HttpRequest {
    pub(crate) verb: HttpVerb,
    pub(crate) protocol: HttpProtocol,
    pub(crate) path: String,
    pub(crate) headers: HashMap<String, String>,
    pub(crate) body: Option<String>,
}

impl HttpRequest {
    pub(crate) fn build(request: &mut dyn BufRead) -> Result<HttpRequest, Box<dyn Error>> {
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

fn build_start_line(
    start_line: String,
) -> Result<(HttpVerb, String, HttpProtocol), Box<dyn Error>> {
    let mut parts = start_line.split(" ");
    if let (Some(verb), Some(path), Some(protocol)) = (parts.next(), parts.next(), parts.next()) {
        let path = path.to_string();
        let verb = HttpVerb::from_str(verb)?;
        let protocol = HttpProtocol::from_str(protocol)?;
        Ok((verb, path, protocol))
    } else {
        Err(Box::new(MalformedRequest {
            error: "invalid start_line".to_string(),
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

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
            request.headers.get("Host"),
            Some(&"example.com".to_string())
        );
        assert_eq!(request.headers.get("User-Agent"), Some(&"test".to_string()));
        assert_eq!(request.body, None);
    }

    #[test]
    fn test_build_failure_invalid_verb() {
        let raw_request =
            b"BAD /api/users HTTP/1.1\r\nHost: example.com\r\nUser-Agent: test\r\n\r\n";
        let mut cursor = Cursor::new(raw_request);

        let result = HttpRequest::build(&mut cursor);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Invalid HTTP verb: 'BAD'")
    }

    #[test]
    fn test_build_failure_invalid_protocol() {
        let raw_request =
            b"GET /api/users HTTP/3.9\r\nHost: example.com\r\nUser-Agent: test\r\n\r\n";
        let mut cursor = Cursor::new(raw_request);

        let result = HttpRequest::build(&mut cursor);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Invalid HTTP verb: 'BAD'")
    }
}
