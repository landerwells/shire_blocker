use std::{fs, io::Error};

pub fn install(ctl: &launchctl::Service) -> Result<(), Error> {
    let plist = format!(
"<?xml version=\"1.0\" encoding=\"UTF-8\"?>
<!DOCTYPE plist PUBLIC \"-//Apple Computer//DTD PLIST 1.0//EN\" \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">
<plist version=\"1.0\">
<dict>
    <key>Label</key>
    <string>{}</string>
    <key>ProgramArguments</key>
    <array>
        <string>{}</string>
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
    <string>/tmp/srhd_sylvanfranklin.out.log</string>
    <key>StandardErrorPath</key>
    <string>/tmp/srhd_sylvanfranklin.err.log</string>
    <key>ProcessType</key>
    <string>Interactive</string>
    <key>Nice</key>
    <integer>-20</integer>
</dict>
</plist>",
        ctl.name,
        // this right here is just ass
        std::env::current_exe().unwrap().to_str().unwrap()
    );

    Ok(fs::write(ctl.plist_path.clone(), plist)?)
}

// out of use currently
// pub fn uninstall(path: String) -> Result<(), Error> {
//     Ok(fs::remove_file(path)?)
// }

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
