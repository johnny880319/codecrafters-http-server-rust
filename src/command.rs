use crate::repl::ParsedRequest;
use crate::template;
use anyhow::Result;
use flate2::{Compression, write::GzEncoder};
use std::{io::Write as _, net::TcpStream};

struct HttpResponse {
    status_line: String,
    headers: Vec<String>,
    body: Vec<u8>,
}

pub fn execute_command(
    parsed_request: ParsedRequest,
    stream: &mut TcpStream,
    dir_path: &str,
) -> Result<bool> {
    let request_method = parsed_request.method.clone();
    let request_target = parsed_request.target.clone();
    let accepts_gzip = parsed_request.accepts_gzip();
    let connection_closed = parsed_request.check_connection_closed();

    let mut response = match (request_method.as_str(), request_target.as_str()) {
        ("GET", "/") => get_root()?,
        ("GET", "/user-agent") => get_user_agent(parsed_request)?,
        ("GET", target) if target.starts_with("/echo/") => get_echo(parsed_request)?,
        ("GET", target) if target.starts_with("/files/") => get_files(parsed_request, dir_path)?,
        ("POST", target) if target.starts_with("/files/") => post_files(parsed_request, dir_path)?,
        _ => not_found()?,
    };

    if accepts_gzip {
        response
            .headers
            .push(template::CONTENT_ENCODING_GZIP.to_string());
        response.body = gzip_compress(&response.body);
    }
    if connection_closed {
        response
            .headers
            .push(template::CONNECTION_CLOSE.to_string());
    }
    response
        .headers
        .push(template::content_length(response.body.len()));

    send_response(stream, response)?;

    Ok(connection_closed)
}

fn gzip_compress(input: &[u8]) -> Vec<u8> {
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(input).expect("gzip compression failed");
    encoder.finish().expect("gzip compression failed")
}

fn send_response(stream: &mut TcpStream, response: HttpResponse) -> Result<()> {
    let mut response_bytes = Vec::new();
    response_bytes.extend_from_slice(response.status_line.as_bytes());
    for header in response.headers {
        response_bytes.extend_from_slice(header.as_bytes());
    }
    response_bytes.extend_from_slice(template::HEADER_END.as_bytes());
    response_bytes.extend_from_slice(&response.body);

    stream.write_all(&response_bytes)?;
    Ok(())
}

fn get_root() -> Result<HttpResponse> {
    let status_line = template::STATUS_200.to_string();
    let headers = Vec::new();
    let body = Vec::new();

    Ok(HttpResponse {
        status_line,
        headers,
        body,
    })
}

fn get_user_agent(parsed_request: ParsedRequest) -> Result<HttpResponse> {
    let status_line = template::STATUS_200.to_string();
    let headers = vec![template::CONTENT_TYPE_PLAIN.to_string()];

    let mut user_agent = String::new();
    for header in parsed_request.headers {
        if let Some(ua) = header.strip_prefix("User-Agent: ") {
            user_agent = ua.to_string();
            break;
        }
    }
    let body = user_agent.into_bytes();

    Ok(HttpResponse {
        status_line,
        headers,
        body,
    })
}

fn get_echo(parsed_request: ParsedRequest) -> Result<HttpResponse> {
    let status_line = template::STATUS_200.to_string();
    let headers = vec![template::CONTENT_TYPE_PLAIN.to_string()];

    let echo_content = parsed_request.target.strip_prefix("/echo/").unwrap_or("");
    let body = echo_content.as_bytes().to_vec();

    Ok(HttpResponse {
        status_line,
        headers,
        body,
    })
}

fn get_files(parsed_request: ParsedRequest, dir_path: &str) -> Result<HttpResponse> {
    let status_line = template::STATUS_200.to_string();
    let headers = vec![template::CONTENT_TYPE_OCTET_STREAM.to_string()];

    let mut file_bytes = Vec::new();
    if let Some(file_path) = parsed_request.target.strip_prefix("/files/") {
        let full_path = format!("{}/{}", dir_path, file_path);
        println!("serving file: {}", full_path);
        match std::fs::read(&full_path) {
            Ok(bytes) => file_bytes = bytes,
            Err(_) => {
                return not_found();
            }
        }
    }

    Ok(HttpResponse {
        status_line,
        headers,
        body: file_bytes,
    })
}

fn post_files(parsed_request: ParsedRequest, dir_path: &str) -> Result<HttpResponse> {
    let status_line = template::STATUS_201.to_string();
    let headers = Vec::new();
    let body = Vec::new();

    if let Some(file_path) = parsed_request.target.strip_prefix("/files/") {
        let full_path = format!("{}/{}", dir_path, file_path);
        println!("serving file: {}", full_path);
        std::fs::write(&full_path, &parsed_request.body)?;
    }

    Ok(HttpResponse {
        status_line,
        headers,
        body,
    })
}

fn not_found() -> Result<HttpResponse> {
    let status_line = template::STATUS_404.to_string();
    let headers = Vec::new();
    let body = Vec::new();

    Ok(HttpResponse {
        status_line,
        headers,
        body,
    })
}
