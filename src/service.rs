use serde::Serialize;
use std::env;
use std::path::PathBuf;
use std::{fs, io::Error};
use std::io;

pub fn install_ctl(ctl: &launchctl::Service) -> Result<(), Error> {
    let exe_path = env::current_exe()?;

    let plist = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>{}</string>
    <key>ProgramArguments</key>
    <array>
        <string>{}</string>
        <string>daemon</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <dict>
        <key>SuccessfulExit</key>
        <false/>
        <key>Crashed</key>
        <true/>
    </dict>
    <key>StandardOutPath</key>
    <string>/tmp/shire.stdout.log</string>
    <key>StandardErrorPath</key>
    <string>/tmp/shire.stderr.log</string>
    <key>ProcessType</key>
    <string>Interactive</string>
    <key>Nice</key>
    <integer>-20</integer>
</dict>
</plist>"#,
        ctl.name,
        exe_path.to_str().ok_or_else(|| std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "shire path is not valid UTF-8"
        ))?,
    );

    fs::write(&ctl.plist_path, plist)?;

    Ok(())
}

pub fn start() -> Result<(), Error> {
    // Get the user's home directory for the plist file
    let home_dir = dirs::home_dir().ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Could not determine home directory",
        )
    })?;

    install_manifest()?;
    if cfg!(target_os = "macos") {
        // Create the proper plist path in the user's LaunchAgents directory
        let plist_path = home_dir
            .join("Library/LaunchAgents")
            .join("com.landerwells.shire.plist");

        let ctl = launchctl::Service::builder()
            .name("com.landerwells.shire")
            .plist_path(plist_path.to_str().unwrap())
            .build();

        install_ctl(&ctl)?;
        ctl.start()?;
    } else {
        let svc = SystemdService {
            name: "shire.service".to_string(),
            service_path: dirs::home_dir()
                .unwrap()
                .join(".config/systemd/user/shire.service"),
        };

        install_systemd(&svc)?;
    }

    Ok(())
}

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

pub struct SystemdService {
    pub name: String,        // e.g. "shire.service"
    pub service_path: PathBuf, // full path to service file
}

pub fn install_systemd(service: &SystemdService) -> Result<(), Error> {
    let exe_path = env::current_exe()?;

    // Create the systemd service unit text
    // Need to add daemon argument to the ExecStart
    let unit_file = format!(
        r#"[Unit]
Description=Shire Daemon
After=network.target

[Service]
ExecStart={}
Restart=always
RestartSec=3
Nice=-20
StandardOutput=append:/tmp/shire.stdout.log
StandardError=append:/tmp/shire.stderr.log

[Install]
WantedBy=default.target
"#,
        exe_path.to_str().ok_or_else(|| io::Error::new(
            io::ErrorKind::InvalidData,
            "shire path is not valid UTF-8"
        ))?,
    );

    // Ensure parent directory exists
    if let Some(parent) = service.service_path.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::write(&service.service_path, unit_file)?;

    Ok(())
}

// Need to complete how to stop the service in MacOS first
pub fn stop() -> Result<(), Error> {
    println!("Stopping the service");

    Ok(())
}
