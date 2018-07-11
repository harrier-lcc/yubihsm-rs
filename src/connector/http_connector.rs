//! Minimalist HTTP client designed for use with yubihsm-connector
//!
//! This is not a full-fledged HTTP client and has been specifically designed
//! to work with yubihsm-connector, which uses HTTP as a wrapper for the
//! underlying YubiHSM encrypted channel protocol.
//!
//! We include this client rather than using a standard crate to minimize
//! dependencies/code surface as well as permit potential usage of this crate
//! in environments (e.g. Intel SGX) where it might be difficult to use a
//! full-fledged HTTP crate (e.g. hyper).

#![allow(unknown_lints, write_with_newline)]

use std::fmt::{self, Write as FmtWrite};
use std::io::{Read, Write as IoWrite};
use std::net::{TcpStream, ToSocketAddrs};
use std::str;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use uuid::Uuid;

use super::{Connector, ConnectorError, Status, USER_AGENT};

/// Default timeouts for reading and writing (5 seconds)
pub const DEFAULT_TIMEOUT_MILLIS: u64 = 5000;

/// Maximum size of the HTTP response from the connector
pub const MAX_RESPONSE_SIZE: usize = 4096;

/// Delimiter string that separates HTTP headers from bodies
const HEADER_DELIMITER: &[u8] = b"\r\n\r\n";

/// HTTP response status indicating success
const HTTP_SUCCESS_STATUS: &str = "HTTP/1.1 200 OK";

/// The Content-Length Header
const CONTENT_LENGTH_HEADER: &str = "Content-Length: ";

/// Configuration options for this connector
#[derive(Debug)]
pub struct HttpConfig {
    /// Address of the connector (IP address or DNS name)
    pub addr: String,

    /// Port the connector process is listening on
    pub port: u16,

    /// Timeout for connecting, reading, and writing in milliseconds
    pub timeout_ms: u64,
}

impl Default for HttpConfig {
    fn default() -> Self {
        Self {
            // Default yubihsm-connector address
            addr: "127.0.0.1".to_owned(),

            // Default yubihsm-connector port
            port: 12345,

            // 5 seconds
            timeout_ms: DEFAULT_TIMEOUT_MILLIS,
        }
    }
}

impl fmt::Display for HttpConfig {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "http://{}:{}", self.addr, self.port)
    }
}

/// HTTP(-ish) connector which supports the minimal parts of the protocol
/// required to communicate with the yubihsm-connector service.
pub struct HttpConnector {
    /// Host we're configured to connect to (i.e. the "Host" HTTP header)
    host: String,

    /// Socket to the connector process
    socket: Arc<Mutex<TcpStream>>,
}

impl Connector for HttpConnector {
    type Config = HttpConfig;

    /// Open a connection to a yubihsm-connector
    fn open(config: Self::Config) -> Result<Self, ConnectorError> {
        let host = format!("{}:{}", config.addr, config.port);

        // Resolve DNS, and for now pick the first available address
        // TODO: round robin DNS support?
        let socketaddr = &host.to_socket_addrs()?.next().ok_or_else(|| {
            connector_err!(
                InvalidURL,
                "no IP addresses in DNS response for {}",
                config.addr
            )
        })?;

        let timeout = Duration::from_millis(DEFAULT_TIMEOUT_MILLIS);
        let socket = TcpStream::connect_timeout(socketaddr, timeout)?;

        socket.set_read_timeout(Some(timeout))?;
        socket.set_write_timeout(Some(timeout))?;

        Ok(Self {
            host,
            socket: Arc::new(Mutex::new(socket)),
        })
    }

    /// GET /connector/status returning the result as connector::Status
    fn status(&self) -> Result<Status, ConnectorError> {
        let http_response = self.get("/connector/status")?;
        Status::parse(str::from_utf8(&http_response)?)
    }

    /// POST /connector/api with a given command message
    fn send_command(&self, uuid: Uuid, cmd: Vec<u8>) -> Result<Vec<u8>, ConnectorError> {
        self.post("/connector/api", uuid, cmd)
    }
}

impl HttpConnector {
    /// Make an HTTP GET request to the yubihsm-connector
    fn get(&self, path: &str) -> Result<Vec<u8>, ConnectorError> {
        let mut request = String::new();

        write!(request, "GET {} HTTP/1.1\r\n", path)?;
        write!(request, "Host: {}\r\n", self.host)?;
        write!(request, "User-Agent: {}\r\n", USER_AGENT)?;
        write!(request, "Content-Length: 0\r\n\r\n")?;

        let mut socket = self.socket.lock().unwrap();
        socket.write_all(request.as_bytes())?;

        Ok(ResponseReader::read(&mut socket)?.into())
    }

    /// Make an HTTP POST request to the yubihsm-connector
    fn post(&self, path: &str, uuid: Uuid, mut body: Vec<u8>) -> Result<Vec<u8>, ConnectorError> {
        let mut headers = String::new();

        write!(headers, "POST {} HTTP/1.1\r\n", path)?;
        write!(headers, "Host: {}\r\n", self.host)?;
        write!(headers, "User-Agent: {}\r\n", USER_AGENT)?;
        write!(headers, "X-Request-ID: {}\r\n", uuid)?;
        write!(headers, "Content-Length: {}\r\n\r\n", body.len())?;

        // It's friendlier to Nagle's algorithm if we combine the request
        // headers and body, especially if the request fits in a single packet
        let mut request: Vec<u8> = headers.into();
        request.append(&mut body);

        let mut socket = self.socket.lock().unwrap();
        socket.write_all(&request)?;

        Ok(ResponseReader::read(&mut socket)?.into())
    }
}

/// Buffered reader for short (i.e. 8kB or less) HTTP responses
struct ResponseReader {
    /// Data buffer
    buffer: [u8; MAX_RESPONSE_SIZE],

    /// Position inside of the data buffer
    pos: usize,

    /// Position at which the body begins
    body_starts_at: Option<usize>,

    /// Length of the body (if we're received it)
    content_length: Option<usize>,
}

impl ResponseReader {
    /// Create a new response buffer
    pub fn read(socket: &mut TcpStream) -> Result<Self, ConnectorError> {
        let mut buffer = Self {
            buffer: [0u8; MAX_RESPONSE_SIZE],
            pos: 0,
            body_starts_at: None,
            content_length: None,
        };

        buffer.read_headers(socket)?;
        buffer.read_body(socket)?;

        Ok(buffer)
    }

    /// Read some data into the internal buffer
    fn fill_buffer(&mut self, socket: &mut TcpStream) -> Result<usize, ConnectorError> {
        let nbytes = socket.read(&mut self.buffer[..])?;
        self.pos += nbytes;
        Ok(nbytes)
    }

    /// Read the HTTP response headers
    fn read_headers(&mut self, socket: &mut TcpStream) -> Result<(), ConnectorError> {
        assert!(self.body_starts_at.is_none(), "already read headers!");

        loop {
            self.fill_buffer(socket)?;

            // Scan the buffer for the header delimiter
            // TODO: this is less efficient than it should be
            let mut offset = 0;
            while self.buffer[offset..].len() > HEADER_DELIMITER.len() {
                if self.buffer[offset..].starts_with(HEADER_DELIMITER) {
                    self.body_starts_at = Some(offset + HEADER_DELIMITER.len());
                    break;
                } else {
                    offset += 1;
                }
            }

            if self.body_starts_at.is_some() {
                break;
            } else if self.pos + 1 >= MAX_RESPONSE_SIZE {
                connector_fail!(
                    ResponseError,
                    "exceeded {}-byte response limit reading headers",
                    MAX_RESPONSE_SIZE
                );
            }
        }

        self.parse_headers()
    }

    /// Parse the HTTP headers, extracting the Content-Length
    fn parse_headers(&mut self) -> Result<(), ConnectorError> {
        let body_starts_at = self.body_starts_at.unwrap();
        let header_str = str::from_utf8(&self.buffer[..body_starts_at])?;

        let mut header_iter = header_str.split("\r\n");

        // Ensure we got a 200 OK status
        match header_iter.next() {
            Some(HTTP_SUCCESS_STATUS) => (),
            Some(status) => connector_fail!(
                ResponseError,
                "unexpected HTTP response status: \"{}\"",
                status
            ),
            None => connector_fail!(ResponseError, "HTTP response status line missing!"),
        }

        for header in header_iter {
            // The only header we care about is "Content-Length"
            if !header.starts_with(CONTENT_LENGTH_HEADER) {
                continue;
            }

            let content_length: usize = header[CONTENT_LENGTH_HEADER.len()..].parse()?;

            if MAX_RESPONSE_SIZE - body_starts_at < content_length {
                connector_fail!(
                    ResponseError,
                    "response body length too large for buffer ({} bytes)",
                    content_length
                );
            }

            self.content_length = Some(content_length);
            return Ok(());
        }

        Err(connector_err!(
            ResponseError,
            "no Content-Length in response!"
        ))
    }

    /// Read the response body into the internal buffer
    fn read_body(&mut self, socket: &mut TcpStream) -> Result<(), ConnectorError> {
        let body_starts_at = self
            .body_starts_at
            .expect("we should've already read the body");

        let content_length = self
            .content_length
            .expect("we should already know the content length");

        while self.pos < body_starts_at + content_length {
            self.fill_buffer(socket)?;
        }

        Ok(())
    }
}

impl Into<Vec<u8>> for ResponseReader {
    fn into(self) -> Vec<u8> {
        let body_starts_at = self
            .body_starts_at
            .expect("we should've already read the body");

        Vec::from(&self.buffer[body_starts_at..self.pos])
    }
}
