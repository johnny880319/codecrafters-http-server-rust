use anyhow::Result;
use std::{
    io::{Read as _, Write as _},
    net::TcpStream,
};

pub fn repl(stream: &mut TcpStream, dir_path: String) -> Result<()> {
    println!("accepted new connection");
    loop {
        let mut buf = [0; 4096];
        let n = stream.read(&mut buf)?;

        if n == 0 {
            println!("connection closed");
            break;
        }

        let request_str = String::from_utf8_lossy(&buf[..n]);
        let request_contents = request_str.split("\r\n").collect::<Vec<_>>();

        println!("received request:");
        for line in &request_contents {
            println!("  {}", line);
        }

        let request_line = request_contents[0];
        let request_target = request_line
            .split(" ")
            .nth(1)
            .ok_or_else(|| anyhow::anyhow!("missing request target"))?;
        let request_headers = &request_contents[1..request_contents.len() - 2];

        if request_target == "/" {
            stream.write_all("HTTP/1.1 200 OK\r\n\r\n".as_bytes())?;
        } else if request_target.starts_with("/echo/") {
            if let Some(echo_content) = request_target.strip_prefix("/echo/") {
                let mut response = String::new();
                response.push_str("HTTP/1.1 200 OK\r\n");
                response.push_str("Content-Type: text/plain\r\n");
                response.push_str(&format!("Content-Length: {}\r\n", echo_content.len()));
                response.push_str("\r\n");
                response.push_str(echo_content);
                stream.write_all(response.as_bytes())?;
            }
        } else if request_target == "/user-agent" {
            let mut echo_content = String::new();
            for header in request_headers {
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
        } else if request_target.starts_with("/file/") {
            let mut file_bytes = Vec::new();
            if let Some(file_path) = request_target.strip_prefix("/file/") {
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
        } else {
            stream.write_all("HTTP/1.1 404 Not Found\r\n\r\n".as_bytes())?;
        }
    }
    Ok(())
}
