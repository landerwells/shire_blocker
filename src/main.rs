// What kinds of commands do I think I will want to send to this?
use std::io::{self, Read, Write};
// use std::convert::TryInto;
use serde::Serialize;
use std::net::TcpListener;

#[derive(Serialize)]
struct Manifest<'a> {
    name: &'a str,
    description: &'a str,
    path: &'a str,
    #[serde(rename = "type")]
    type_field: &'a str,
    allowed_extensions: [&'a str; 1],
}

fn main() {
    // let path: PathBuf = get_manifest_dir();
    if let Err(e) = install_manifest() {
        eprintln!("Failed to install manifest: {e}");
        std::process::exit(1);
    } else {
        println!("Manifest installed successfully.");
    }

    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    let (mut stream, _) = listener.accept().unwrap();
    let mut buffer = [0; 512];
    stream.read_exact(&mut buffer).unwrap();
    println!("Received: {}", String::from_utf8_lossy(&buffer));
}

use std::fs;
use std::path::PathBuf;

fn get_manifest_dir() -> PathBuf {
    if cfg!(target_os = "macos") {
        dirs::home_dir()
            .unwrap()
            .join("Library/Application Support/Mozilla/NativeMessagingHosts")
    } else if cfg!(target_os = "windows") {
        dirs::data_dir()
            .unwrap()
            .join("Mozilla")
            .join("NativeMessagingHosts")
    } else {
        dirs::home_dir()
            .unwrap()
            .join(".mozilla/native-messaging-hosts")
    }
}

fn install_manifest() -> std::io::Result<()> {
    let manifest_dir = get_manifest_dir();
    fs::create_dir_all(&manifest_dir)?;

    // TODO: No longer the correct path, need to get bridge instead
    // let exe_path = std::env::current_exe()?.display().to_string();
    let bridge_path = std::env::current_exe()?
        .parent()
        .unwrap()
        .join("shire_bridge")
        .display()
        .to_string();

    let manifest = Manifest {
        name: "com.shire_blocker",
        description: "Shire Blocker",
        path: &bridge_path,
        type_field: "stdio",
        allowed_extensions: ["shire_blocker@example.com"],
    };

    let manifest_json = serde_json::to_string_pretty(&manifest).unwrap();
    let manifest_json = manifest_json.replace("type_field", "type");
    let manifest_path = manifest_dir.join("com.shire_blocker.json");
    fs::write(manifest_path, manifest_json)?;
    Ok(())
}
