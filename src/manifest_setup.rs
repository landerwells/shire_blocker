use std::fs;
use serde::Serialize;
use std::path::PathBuf;

#[derive(Serialize)]
struct Manifest<'a> {
    name: &'a str,
    description: &'a str,
    path: &'a str,
    #[serde(rename = "type")]
    type_field: &'a str,
    allowed_extensions: [&'a str; 1],
}

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

pub fn install_manifest() -> std::io::Result<()> {
    let manifest_dir = get_manifest_dir();
    fs::create_dir_all(&manifest_dir)?;

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
