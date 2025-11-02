use crate::HttpCode;
use std::fmt;

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

#[cfg(test)]
mod tests {}
