mod commands;
mod service;
use clap::{Parser, Subcommand};
use commands::list_available_blocks;
use std::os::unix::net::UnixStream;

#[derive(Parser)]
#[command(
    name = "shire",
    version = "1.0",
    author = "Lander Wells",
    about = "A tool for managing blocks and services"
)]
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
    /// Manage the shire service
    Service {
        #[command(subcommand)]
        action: ServiceAction,
    },
}

#[derive(Subcommand)]
enum BlockAction {
    /// List all available blocks
    List,
    /// Start a block
    Start {
        name: String,
        #[arg(long)]
        lock: Option<String>, // e.g. duration
    },
    /// Stop a block
    Stop { name: String },
}

#[derive(Subcommand)]
enum ServiceAction {
    /// Start the shire service (install and start daemon)
    Start,
    /// Stop the shire service
    Stop,
    /// Restart the shire service
    Restart,
}

const SOCKET_PATH: &str = "/tmp/shire_cli.sock";

// Refactor a bit since I don't like how it is handled along with the main function
fn setup_socket() -> Result<UnixStream, std::io::Error> {
    // Attempt to connect to the Unix socket
    match UnixStream::connect(SOCKET_PATH) {
        Ok(stream) => Ok(stream),
        Err(e) => {
            eprintln!("Failed to connect to the shire service socket at {SOCKET_PATH}: {e}");
            Err(e)
        }
    }
}

fn main() {
    let args = Args::parse();

    match args.command {
        Commands::Block { action } => match action {
            BlockAction::List => {
                println!("Listing all available blocks...");
                let mut cli_sock = setup_socket().expect("Failed to connect to the shire service socket");
                list_available_blocks(&mut cli_sock).expect("Failed to list available blocks");
            }
            BlockAction::Start { name, lock } => {
                println!("Starting block: {}", name);
                if let Some(lock_duration) = lock {
                    println!("Lock duration: {}", lock_duration);
                }
                // TODO: Implement block start
            }
            BlockAction::Stop { name } => {
                println!("Stopping block: {}", name);
                // TODO: Implement block stop
            }
        },
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
                // TODO: Implement service stop
            }
            ServiceAction::Restart => {
                println!("Restarting shire service...");
                // TODO: Implement service restart
            }
        },
    }
}
