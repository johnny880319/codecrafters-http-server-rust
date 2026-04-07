use crate::command;
use anyhow::Result;
use std::{io::Read as _, net::Shutdown::Both, net::TcpStream};

pub fn repl(stream: &mut TcpStream, dir_path: String) -> Result<()> {
    println!("accepted new connection");
    loop {
        let mut buf = [0; 4096];
        let n = stream.read(&mut buf)?;

        let parsed_request = parse_request(&buf, n)?;

        let connection_closed = command::execute_command(parsed_request, stream, dir_path.clone())?;
        if connection_closed {
            println!("connection closed by client");
            stream.shutdown(Both)?;
            break;
        }
    }
    Ok(())
}

pub struct ParsedRequest {
    pub method: String,
    pub target: String,
    pub headers: Vec<String>,
    pub body: String,
}

fn parse_request(buf: &[u8], n: usize) -> Result<ParsedRequest> {
    let request_str = String::from_utf8_lossy(&buf[..n]);
    let request_contents = request_str.split("\r\n").collect::<Vec<_>>();

    println!("received request:");
    for line in &request_contents {
        println!("  {}", line);
    }

    let request_line = request_contents[0];
    let method = request_line
        .split(" ")
        .next()
        .ok_or_else(|| anyhow::anyhow!("missing request method"))?
        .to_string();
    let target = request_line
        .split(" ")
        .nth(1)
        .ok_or_else(|| anyhow::anyhow!("missing request target"))?
        .to_string();
    let headers: Vec<String> = request_contents[1..request_contents.len() - 2]
        .iter()
        .map(|s| s.to_string())
        .collect();
    let body = request_contents.last().unwrap_or(&"").to_string();
    Ok(ParsedRequest {
        method,
        target,
        headers,
        body,
    })
}
