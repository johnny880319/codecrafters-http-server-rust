use std::net::TcpListener;
mod repl;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        // Hi
        match stream {
            Ok(mut stream) => {
                println!("accepted new connection");
                repl::repl(&mut stream);
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
