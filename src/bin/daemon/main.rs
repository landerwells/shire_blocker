use std::io::Read;
use std::net::{TcpListener, TcpStream};

mod config;

fn main() {
    // config::parse_config();

    // Further down we will need some branching to listen to multiple things
    // - config file

    // - messages from bridge

    // Read characters one letter at a time?
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();

    // let (mut stream, _) = listener.accept().unwrap();
    
    for stream in listener.incoming() {
        handle_client(&mut stream.unwrap());
    }

}

fn handle_client(stream: &mut TcpStream) {
    let mut buffer = [0; 512];

    loop {
        match stream.read(&mut buffer) {
            Ok(0) => break, // Connection closed
            Ok(n) => {
                // Process the received data
                println!("Received {n} bytes");
                // handle_message(&mut stream, &buffer[..n]);
            }
            Err(e) => {
                eprintln!("Failed to read from stream: {e}");
                break;
            }
        }
    }
    println!("Received: {}", String::from_utf8_lossy(&buffer));

}
