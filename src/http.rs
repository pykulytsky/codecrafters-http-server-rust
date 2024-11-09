use std::io::Write;
use std::{collections::BTreeMap, io::BufRead};

use crate::error::HttpError;

#[derive(Debug)]
pub struct HttpRequest<'req> {
    pub method: HttpMethod,
    pub url: &'req str,
    pub headers: BTreeMap<&'req str, &'req str>,
    // headers: &'req [u8],
    pub body: Option<&'req [u8]>,
}

#[derive(Debug, PartialEq)]
pub enum HttpMethod {
    Get,
    Post,
    Patch,
    Put,
    Option,
}

impl HttpMethod {
    pub fn as_bytes(&self) -> &'static [u8] {
        match self {
            HttpMethod::Get => b"GET",
            HttpMethod::Post => b"POST",
            HttpMethod::Patch => b"PATCH",
            HttpMethod::Put => b"PUT",
            HttpMethod::Option => b"OPTION",
        }
    }

    pub fn encode(input: &[u8]) -> Result<Self, HttpError> {
        match input {
            b"GET" => Ok(Self::Get),
            b"POST" => Ok(Self::Post),
            b"PATCH" => Ok(Self::Patch),
            b"PUT" => Ok(Self::Put),
            b"OPTION" => Ok(Self::Option),
            _ => Err(HttpError::MethodError),
        }
    }
}

impl<'req> HttpRequest<'req> {
    pub fn decode(input: &'req [u8]) -> Result<Self, HttpError> {
        let mut lines = input
            .split(|b| b == &0xA)
            .map(|line| line.strip_suffix(&[0xD]).unwrap_or(line));
        let first_line = lines.next().ok_or(HttpError::ProtocolError)?;
        let mut first_line_elements = first_line.split(|b| *b == b' ');
        let method =
            HttpMethod::encode(first_line_elements.next().ok_or(HttpError::ProtocolError)?)?;
        let url = std::str::from_utf8(first_line_elements.next().ok_or(HttpError::ProtocolError)?)
            .map_err(|_| HttpError::UrlError)?;
        // TODO: the rest of the line
        let mut headers = BTreeMap::new();
        for line in lines.by_ref() {
            if line == b"\r\n" || line == b"" {
                break;
            }
            let header_line = std::str::from_utf8(line).map_err(|_| HttpError::HeaderError)?;
            let mut parts = header_line.split(": ");
            headers.insert(
                parts.next().ok_or(HttpError::HeaderError)?,
                parts.next().ok_or(HttpError::HeaderError)?,
            );
        }
        let body = lines.next();

        Ok(Self {
            method,
            url,
            body,
            headers,
        })
    }

    pub fn encode(self) -> Vec<u8> {
        let mut buf = vec![];
        buf.extend(self.method.as_bytes());
        buf.push(b' ');
        buf.extend(self.url.as_bytes());
        buf.push(b' ');
        buf.extend(b"HTTP/1.1");
        buf.extend(b"\r\n");
        // TODO headers
        buf.extend(b"\r\n");
        buf.extend(b"\r\n");
        if let Some(body) = self.body {
            buf.extend(body);
        }

        buf
    }
}

#[derive(Debug)]
pub struct HttpResponse {
    pub status_code: HttpStatusCode,
    pub body: Option<Vec<u8>>,
    pub headers: Option<BTreeMap<String, String>>,
}

impl HttpResponse {
    pub fn new(status_code: HttpStatusCode) -> Self {
        Self {
            status_code,
            body: None,
            headers: None,
        }
    }

    pub fn new_ok() -> Self {
        Self {
            status_code: HttpStatusCode::Ok,
            body: None,
            headers: None,
        }
    }

    pub fn new_created() -> Self {
        Self {
            status_code: HttpStatusCode::Created,
            body: None,
            headers: None,
        }
    }

    pub fn new_not_found() -> Self {
        Self {
            status_code: HttpStatusCode::NotFound,
            body: None,
            headers: None,
        }
    }

    pub fn with_body(mut self, body: Vec<u8>) -> Self {
        self.body = Some(body);
        self
    }

    pub fn set_header(mut self, key: &str, value: &str) -> Self {
        if let Some(headers) = self.headers.as_mut() {
            headers.insert(key.to_string(), value.to_string());
        } else {
            let mut headers = BTreeMap::new();
            headers.insert(key.to_string(), value.to_string());
            self.headers = Some(headers);
        }
        self
    }

    pub fn encode(self) -> Vec<u8> {
        let mut buf = vec![];

        buf.extend(b"HTTP/1.1 ");
        buf.extend(self.status_code.as_bytes());
        buf.extend(b"\r\n");
        // buf.extend(b"Content-Type: text/plain\r\n");
        buf.extend(b"Content-Length: ");
        let content_length = self.body.as_ref().map(|body| body.len()).unwrap_or(0);
        write!(buf, "{}", content_length).unwrap();
        buf.extend(b"\r\n");
        if let Some(headers) = self.headers {
            for (key, value) in headers {
                buf.extend(key.as_bytes());
                buf.extend(b": ");
                buf.extend(value.as_bytes());
                buf.extend(b"\r\n")
            }
        } else {
            buf.extend(b"Content-Type: text/plain\r\n");
        }
        buf.extend(b"\r\n");
        if let Some(body) = self.body {
            buf.extend(body);
        }

        buf
    }
}

#[derive(Debug)]
pub enum HttpStatusCode {
    Ok,
    NotFound,
    Created,
}

impl HttpStatusCode {
    pub fn as_bytes(&self) -> &'static [u8] {
        match self {
            HttpStatusCode::Ok => b"200 OK",
            HttpStatusCode::NotFound => b"404 Not Found",
            HttpStatusCode::Created => b"201 Created",
        }
    }
}
