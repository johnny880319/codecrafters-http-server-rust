use anyhow::Result;
use std::{
    io::{Read as _, Write as _},
    net::TcpStream,
};

pub fn repl(stream: &mut TcpStream) -> Result<()> {
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

        if request_target == "/" {
            stream.write_all("HTTP/1.1 200 OK\r\n\r\n".as_bytes())?;
        } else {
            stream.write_all("HTTP/1.1 404 Not Found\r\n\r\n".as_bytes())?;
        }
    }
    Ok(())
}
