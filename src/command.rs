use crate::repl::ParsedRequest;
use anyhow::{Result, bail};
use std::{io::Write as _, net::TcpStream};

pub fn execute_command(
    parsed_request: ParsedRequest,
    stream: &mut TcpStream,
    dir_path: String,
) -> Result<()> {
    let request_method = parsed_request.method.as_str();
    let request_target = parsed_request.target.as_str();
    match (request_method, request_target) {
        ("GET", "/") => {
            get_cmd(stream)
        }
        ("GET", target) if target.starts_with("/echo/") => {
            get_echo(parsed_request, stream)
        }
        ("GET", "/user-agent") => {
            get_user_agent(parsed_request, stream)
        }
        ("GET", target) if target.starts_with("/files/") => {
            get_files(parsed_request, stream, dir_path)
        }
        ("POST", target) if target.starts_with("/files/") => {
            post_files(parsed_request, stream, dir_path)
        }
        _ => {
            not_found(stream)
        }
    }
}

fn get_cmd(stream: &mut TcpStream) -> Result<()> {
    stream.write_all("HTTP/1.1 200 OK\r\n\r\n".as_bytes())?;
    Ok(())
}

fn get_echo(parsed_request: ParsedRequest, stream: &mut TcpStream) -> Result<()> {
    if let Some(echo_content) = parsed_request.target.strip_prefix("/echo/") {
        let mut response = String::new();
        response.push_str("HTTP/1.1 200 OK\r\n");
        response.push_str("Content-Type: text/plain\r\n");
        response.push_str(&format!("Content-Length: {}\r\n", echo_content.len()));
        response.push_str("\r\n");
        response.push_str(echo_content);
        stream.write_all(response.as_bytes())?;
        return Ok(());
    }
    bail!("invalid echo request");
}

fn get_user_agent(parsed_request: ParsedRequest, stream: &mut TcpStream) -> Result<()> {
    let mut echo_content = String::new();
    for header in parsed_request.headers {
        if let Some(user_agent) = header.strip_prefix("User-Agent: ") {
            echo_content = user_agent.to_string();
            break;
        }
    }
    let mut response = String::new();
    response.push_str("HTTP/1.1 200 OK\r\n");
    response.push_str("Content-Type: text/plain\r\n");
    response.push_str(&format!("Content-Length: {}\r\n", echo_content.len()));
    response.push_str("\r\n");
    response.push_str(&echo_content);
    stream.write_all(response.as_bytes())?;
    Ok(())
}

fn get_files(
    parsed_request: ParsedRequest,
    stream: &mut TcpStream,
    dir_path: String,
) -> Result<()> {
    let mut file_bytes = Vec::new();
    if let Some(file_path) = parsed_request.target.strip_prefix("/files/") {
        let full_path = format!("{}/{}", dir_path, file_path);
        println!("serving file: {}", full_path);
        match std::fs::read(&full_path) {
            Ok(bytes) => file_bytes = bytes,
            Err(_) => {
                stream.write_all("HTTP/1.1 404 Not Found\r\n\r\n".as_bytes())?;
                return Ok(());
            }
        }
    }

    let mut response = String::new();
    response.push_str("HTTP/1.1 200 OK\r\n");
    response.push_str("Content-Type: application/octet-stream\r\n");
    response.push_str(&format!("Content-Length: {}\r\n", file_bytes.len()));
    response.push_str("\r\n");
    response.push_str(&String::from_utf8_lossy(&file_bytes));
    stream.write_all(response.as_bytes())?;
    Ok(())
}

fn post_files(
    parsed_request: ParsedRequest,
    stream: &mut TcpStream,
    dir_path: String,
) -> Result<()> {
    if let Some(file_path) = parsed_request.target.strip_prefix("/files/") {
        let full_path = format!("{}/{}", dir_path, file_path);
        println!("serving file: {}", full_path);
        std::fs::write(&full_path, parsed_request.body)?;
    }

    let response = "HTTP/1.1 201 Created\r\n\r\n".to_string();
    stream.write_all(response.as_bytes())?;
    Ok(())
}

fn not_found(stream: &mut TcpStream) -> Result<()> {
    stream.write_all("HTTP/1.1 404 Not Found\r\n\r\n".as_bytes())?;
    Ok(())
}
