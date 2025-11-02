use std::fmt;
use std::fmt::Display;
use std::str::FromStr;

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
