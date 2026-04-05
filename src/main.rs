use std::net::TcpListener;
mod repl;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                println!("accepted new connection");
                let _ = repl::repl(&mut stream);
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
