// Status line
pub const STATUS_200: &str = "HTTP/1.1 200 OK\r\n";
pub const STATUS_201: &str = "HTTP/1.1 201 Created\r\n";
pub const STATUS_404: &str = "HTTP/1.1 404 Not Found\r\n";

// Headers
pub const CONTENT_TYPE_PLAIN: &str = "Content-Type: text/plain\r\n";
pub const CONTENT_TYPE_OCTET_STREAM: &str = "Content-Type: application/octet-stream\r\n";
pub const CONTENT_ENCODING_GZIP: &str = "Content-Encoding: gzip\r\n";
pub const CONNECTION_CLOSE: &str = "Connection: close\r\n";

pub fn content_length(length: usize) -> String {
    format!("Content-Length: {length}\r\n")
}

pub const HEADER_END: &str = "\r\n";
