use std::io::Read;
use std::net::TcpListener;
mod manifest_setup;

fn main() {
    // Likely will want a setup function to install the manifest

    if let Err(e) = manifest_setup::install_manifest() {
        eprintln!("Failed to install manifest: {e}");
        std::process::exit(1);
    } else {
        println!("Manifest installed successfully.");
    }

    // Further down we will need some branching to listen to multiple things
    // - config file
    // - messages from bridge

    // Read characters one letter at a time?
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();

    let (mut stream, _) = listener.accept().unwrap();
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

