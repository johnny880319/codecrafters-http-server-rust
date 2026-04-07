use std::{env, net::TcpListener, thread};
mod command;
mod connection;
mod template;

fn main() {
    // parse the --directory argument
    let cmd_args = env::args().collect::<Vec<String>>();
    let dir_pos = cmd_args.iter().position(|arg| arg == "--directory");
    let dir_path = dir_pos.map_or_else(|| ".".to_string(), |pos| cmd_args[pos + 1].clone());

    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                println!("accepted new connection");
                let dir_path = dir_path.clone();
                thread::spawn(move || {
                    let _ = connection::handle_connection(&mut stream, dir_path);
                });
            }
            Err(e) => {
                println!("error: {e}");
            }
        }
    }
}
