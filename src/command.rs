use crate::repl::ParsedRequest;
use crate::template;
use anyhow::{Result, bail};
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
    dir_path: String,
) -> Result<bool> {
    let request_method = parsed_request.method.as_str();
    let request_target = parsed_request.target.as_str();
    let accepts_gzip = check_is_gzip(&parsed_request);
    let connection_closed = check_connection_closed(&parsed_request);

    let mut response;

    match (request_method, request_target) {
        ("GET", "/") => {
            response = get_cmd()?;
        }
        ("GET", target) if target.starts_with("/echo/") => {
            response = get_echo(parsed_request)?;
        }
        ("GET", "/user-agent") => {
            response = get_user_agent(parsed_request)?;
        }
        ("GET", target) if target.starts_with("/files/") => {
            response = get_files(parsed_request, dir_path)?;
        }
        ("POST", target) if target.starts_with("/files/") => {
            response = post_files(parsed_request, dir_path)?;
        }
        _ => {
            response = not_found()?;
        }
    }

    if accepts_gzip {
        response
            .headers
            .push("Content-Encoding: gzip\r\n".to_string());
        response.body = gzip_compress(&response.body);
    }
    if connection_closed {
        response.headers.push("Connection: close\r\n".to_string());
    };

    send_response(stream, response)?;

    Ok(connection_closed)
}

fn check_is_gzip(parsed_request: &ParsedRequest) -> bool {
    for header in &parsed_request.headers {
        if let Some(accept_encoding) = header.strip_prefix("Accept-Encoding: ")
            && accept_encoding.split(", ").any(|e| e == "gzip")
        {
            return true;
        }
    }
    false
}

fn check_connection_closed(request: &ParsedRequest) -> bool {
    for header in &request.headers {
        if header == "Connection: close" {
            return true;
        }
    }
    false
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

fn get_cmd() -> Result<HttpResponse> {
    let status_line = template::STATUS_200.to_string();
    let headers: Vec<String> = vec![];
    let body = "".as_bytes().to_vec();

    Ok(HttpResponse {
        status_line,
        headers,
        body,
    })
}

fn get_echo(parsed_request: ParsedRequest) -> Result<HttpResponse> {
    let status_line = template::STATUS_200.to_string();
    let mut headers = vec![template::CONTENT_TYPE_PLAIN.to_string()];
    let body;

    if let Some(echo_content) = parsed_request.target.strip_prefix("/echo/") {
        headers.push(template::content_length(echo_content.len()));
        body = echo_content.as_bytes().to_vec();

        return Ok(HttpResponse {
            status_line,
            headers,
            body,
        });
    }
    bail!("invalid echo request");
}

fn get_user_agent(parsed_request: ParsedRequest) -> Result<HttpResponse> {
    let status_line = template::STATUS_200.to_string();
    let mut headers = vec![template::CONTENT_TYPE_PLAIN.to_string()];

    let mut user_agent = String::new();
    for header in parsed_request.headers {
        if let Some(ua) = header.strip_prefix("User-Agent: ") {
            user_agent = ua.to_string();
            break;
        }
    }
    headers.push(template::content_length(user_agent.len()));
    let body = user_agent.as_bytes().to_vec();

    Ok(HttpResponse {
        status_line,
        headers,
        body,
    })
}

fn get_files(parsed_request: ParsedRequest, dir_path: String) -> Result<HttpResponse> {
    let status_line = template::STATUS_200.to_string();
    let mut headers = vec![template::CONTENT_TYPE_OCTET_STREAM.to_string()];

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

    headers.push(template::content_length(file_bytes.len()));
    let body = file_bytes;

    Ok(HttpResponse {
        status_line,
        headers,
        body,
    })
}

fn post_files(parsed_request: ParsedRequest, dir_path: String) -> Result<HttpResponse> {
    let status_line = template::STATUS_201.to_string();
    let headers: Vec<String> = vec![];
    let body = "".as_bytes().to_vec();

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
    let headers: Vec<String> = vec![];
    let body = "".as_bytes().to_vec();

    Ok(HttpResponse {
        status_line,
        headers,
        body,
    })
}
