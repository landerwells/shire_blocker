mod commands;
mod service;
mod daemon;
mod config;
use serde_json::json;
use std::collections::HashMap;
use clap::{Parser, Subcommand};
use commands::list_blocks;
use std::os::unix::net::UnixStream;

use crate::commands::*;
use crate::daemon::start_daemon;

#[derive(Parser)]
#[command(
    name = "shire",
    version = "1.0",
    author = "Lander Wells",
    about = "A tool for managing blocks and services"
)]
// #[clap(disable_help_flag = true)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Manage blocks
    Block {
        #[command(subcommand)]
        action: BlockAction,
    },
    /// Manage the block scheduler
    Schedule {
        #[command(subcommand)]
        action: ScheduleAction,
    },
    /// Manage the shire service
    Service {
        #[command(subcommand)]
        action: ServiceAction,
    },
    /// Launch the daemon
    Daemon {
        #[arg(long)]
        config: Option<String>, // e.g. duration
    }
}

#[derive(Subcommand)]
enum BlockAction {
    /// List all available blocks
    List,
    /// Start a block
    Start {
        name: String,
        #[arg(long)]
        // Need to make this more obvious to users
        lock: Option<String>, // e.g. duration
    },
    /// Stop a block
    Stop { name: String },
    Lock { 
        name: String ,
        #[arg(long)]
        // Need to make this more obvious to users
        lock: Option<String>, // e.g. duration
    }
}

#[derive(Subcommand)]
enum ScheduleAction {
    /// List the schedule
    List,
}

#[derive(Subcommand)]
enum ServiceAction {
    /// Start the shire service (install and start daemon)
    Start,
    /// Stop the shire service
    Stop,
    /// Restart the shire service
    Restart,
    /// Uninstall the shire service and clean up all files
    Uninstall,
}

const SOCKET_PATH: &str = "/tmp/shire_cli.sock";

fn main() {
    let args = Args::parse();

    match args.command {
        Commands::Block { action } => match action {
            BlockAction::List => {
                let mut stream = UnixStream::connect(SOCKET_PATH).expect("Failed to connect to the shire service socket at {SOCKET_PATH}: {e}");
                list_blocks(&mut stream).expect("Failed to list available blocks");
            }
            BlockAction::Start { name, lock } => {
                let mut stream = UnixStream::connect(SOCKET_PATH).expect("Failed to connect to the shire service socket at {SOCKET_PATH}: {e}");

                let mut params = HashMap::new();
                params.insert("name", json!(name));
                if let Some(lock_str) = lock {
                    params.insert("lock", json!(lock_str));
                }
                send_action_with_params(&mut stream, "start_block", Some(params)).unwrap();
            }
            BlockAction::Stop { name } => {
                let mut stream = UnixStream::connect(SOCKET_PATH).expect("Failed to connect to the shire service socket at {SOCKET_PATH}: {e}");

                let mut params = HashMap::new();
                params.insert("name", json!(name));
                send_action_with_params(&mut stream, "stop_block", Some(params)).unwrap();
            }
            BlockAction::Lock { name, lock } => {
                let mut stream = UnixStream::connect(SOCKET_PATH).expect("Failed to connect to the shire service socket at {SOCKET_PATH}: {e}");

                let mut params = HashMap::new();
                params.insert("name", json!(name));
                params.insert("lock", json!(lock));
                send_action_with_params(&mut stream, "lock_block", Some(params)).unwrap();
            }
        },
        Commands::Schedule { action } => match action {
            ScheduleAction::List => {
                println!("Listing the block schedule...");
            }
        }
        Commands::Service { action } => match action {
            ServiceAction::Start => {
                println!("Starting shire service (install and start daemon)...");
                match service::start() {
                    Ok(_) => println!("Shire service started successfully."),
                    Err(e) => eprintln!("Failed to start shire service: {e}"),
                }
            }
            ServiceAction::Stop => {
                println!("Stopping shire service...");
                match service::stop() {
                    Ok(_) => println!("Shire service stopped successfully."),
                    Err(e) => eprintln!("Failed to stop shire service: {e}"),
                }
            }
            ServiceAction::Restart => {
                println!("Restarting shire service...");
                match service::stop() {
                    Ok(_) => {
                        println!("Shire service stopped successfully.");
                        match service::start() {
                            Ok(_) => println!("Shire service restarted successfully."),
                            Err(e) => eprintln!("Failed to start shire service after stop: {e}"),
                        }
                    }
                    Err(e) => eprintln!("Failed to stop shire service: {e}"),
                }
            }
            ServiceAction::Uninstall => {
                println!("Uninstalling shire service...");
                match service::uninstall() {
                    Ok(_) => println!("Shire service uninstalled successfully."),
                    Err(e) => eprintln!("Failed to uninstall shire service: {e}"),
                }
            }
        },
        Commands::Daemon { config } => {
            // I want an optional parameter to specify the config file
            println!("Starting the daemon with config {config:?}");
            start_daemon();
        }
    }
}
