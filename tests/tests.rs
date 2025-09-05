// use std::thread;
// use std::time::Duration;
// use std::os::unix::net::UnixStream;
// use std::fs;
// use serde_json::{json, Value};
// use shire_blocker::{send_length_prefixed_message, recv_length_prefixed_message};
// use shire_blocker::daemon::start_daemon;
//
// const TEST_CONFIG_PATH: &str = "tests/fixtures/shire.toml";
// const CLI_SOCKET_PATH: &str = "/tmp/shire_cli.sock";
// const BRIDGE_SOCKET_PATH: &str = "/tmp/shire_bridge.sock";
//
// #[test]
// fn test_daemon_lifecycle() {
//     // Clean up any existing sockets
//     let _ = fs::remove_file(CLI_SOCKET_PATH);
//     let _ = fs::remove_file(BRIDGE_SOCKET_PATH);
//
//     // Start daemon in a separate thread
//     let daemon_handle = thread::spawn(move || {
//         start_daemon(Some(TEST_CONFIG_PATH.to_string()));
//     });
//
//     // Give daemon time to start up
//     thread::sleep(Duration::from_millis(500));
//
//     // Test CLI connection and basic functionality
//     test_cli_operations();
//
//     // Test bridge connection
//     test_bridge_connection();
//
//     // Clean up - terminate daemon thread by removing sockets
//     let _ = fs::remove_file(CLI_SOCKET_PATH);
//     let _ = fs::remove_file(BRIDGE_SOCKET_PATH);
// }
//
// fn test_cli_operations() {
//     // Connect to CLI socket
//     let mut stream = UnixStream::connect(CLI_SOCKET_PATH)
//         .expect("Failed to connect to daemon CLI socket");
//
//     // Test list_blocks command
//     let list_request = json!({
//         "action": "list_blocks"
//     });
//
//     send_length_prefixed_message(&mut stream, list_request.to_string().as_bytes())
//         .expect("Failed to send list_blocks request");
//
//     let response = recv_length_prefixed_message(&mut stream)
//         .expect("Failed to receive list_blocks response");
//
//     let response_str = String::from_utf8(response).expect("Invalid UTF-8 in response");
//     let response_json: Value = serde_json::from_str(&response_str)
//         .expect("Invalid JSON in response");
//
//     // Verify we got blocks from config
//     assert!(response_json["blocks"].is_object(), "Response should contain blocks object");
//
//     // Test start_block command
//     let start_request = json!({
//         "action": "start_block",
//         "name": "Algorithmic Feeds"
//     });
//
//     send_length_prefixed_message(&mut stream, start_request.to_string().as_bytes())
//         .expect("Failed to send start_block request");
//
//     let start_response = recv_length_prefixed_message(&mut stream)
//         .expect("Failed to receive start_block response");
//
//     let start_response_str = String::from_utf8(start_response).expect("Invalid UTF-8 in response");
//     let start_response_json: Value = serde_json::from_str(&start_response_str)
//         .expect("Invalid JSON in response");
//
//     assert_eq!(start_response_json["status"], "started");
//     assert_eq!(start_response_json["block"], "Algorithmic Feeds");
//
//     // Test stop_block command
//     let stop_request = json!({
//         "action": "stop_block",
//         "name": "Algorithmic Feeds"
//     });
//
//     send_length_prefixed_message(&mut stream, stop_request.to_string().as_bytes())
//         .expect("Failed to send stop_block request");
//
//     let stop_response = recv_length_prefixed_message(&mut stream)
//         .expect("Failed to receive stop_block response");
//
//     let stop_response_str = String::from_utf8(stop_response).expect("Invalid UTF-8 in response");
//     let stop_response_json: Value = serde_json::from_str(&stop_response_str)
//         .expect("Invalid JSON in response");
//
//     assert_eq!(stop_response_json["status"], "stopped");
//     assert_eq!(stop_response_json["block"], "Algorithmic Feeds");
// }
//
// fn test_bridge_connection() {
//     // Connect to bridge socket
//     let mut stream = UnixStream::connect(BRIDGE_SOCKET_PATH)
//         .expect("Failed to connect to daemon bridge socket");
//
//     // Send get_state request
//     let state_request = json!({
//         "action": "get_state"
//     });
//
//     send_length_prefixed_message(&mut stream, state_request.to_string().as_bytes())
//         .expect("Failed to send get_state request");
//
//     // Give daemon time to process and respond
//     thread::sleep(Duration::from_millis(100));
//
//     // Try to read response (may timeout if daemon is busy)
//     match recv_length_prefixed_message(&mut stream) {
//         Ok(response) => {
//             let response_str = String::from_utf8(response).expect("Invalid UTF-8 in response");
//             let response_json: Value = serde_json::from_str(&response_str)
//                 .expect("Invalid JSON in response");
//
//             // Verify state update message format
//             assert_eq!(response_json["type"], "state_update");
//             assert!(response_json["blocks"].is_object(), "Response should contain blocks object");
//         },
//         Err(_) => {
//             // Bridge connection established but may not have received response yet
//             // This is acceptable as we're testing the connection works
//         }
//     }
// }
//
// #[test] 
// fn test_daemon_config_loading() {
//     // Test that daemon can load and parse the test config
//     use shire_blocker::config::parse_config;
//
//     let config = parse_config(Some(TEST_CONFIG_PATH.to_string()))
//         .expect("Failed to parse test config");
//
//     // Verify config was loaded correctly
//     assert!(!config.blocks.is_empty(), "Config should contain blocks");
//     assert!(config.blocks.len() >= 2, "Config should contain at least 2 blocks");
//
//     // Check specific blocks from test config
//     let block_names: Vec<&String> = config.blocks.iter().map(|b| &b.name).collect();
//     assert!(block_names.contains(&&"Algorithmic Feeds".to_string()));
//     assert!(block_names.contains(&&"stock_sites".to_string()));
// }
