mod commands;
mod service;
use clap::{Parser, Subcommand};
use commands::list_blocks;
use std::os::unix::net::UnixStream;

use crate::commands::*;

#[derive(Parser)]
#[command(
    name = "shire",
    version = "1.0",
    author = "Lander Wells",
    about = "A tool for managing blocks and services"
)]
#[clap(disable_help_flag = true)]
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
}

const SOCKET_PATH: &str = "/tmp/shire_cli.sock";

// Refactor a bit since I don't like how it is handled along with the main function,
// but I do like this error message.
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
                // I would also like people to be able to do shire block ls
                // println!("Listing all available blocks...");
                let mut stream = setup_socket().expect("Failed to connect to the shire service socket");
                list_blocks(&mut stream).expect("Failed to list available blocks");
            }
            BlockAction::Start { name, lock } => {
                let mut stream = setup_socket().expect("Failed to connect to the shire service socket");

                start_block(&mut stream, name, lock).expect("Failed to start block");
            }
            BlockAction::Stop { name } => {
                let mut stream = setup_socket().expect("Failed to connect to the shire service socket");

                stop_block(&mut stream, name).expect("Failed to stop block");
            }
            BlockAction::Lock { name, lock } => {
                let mut stream = setup_socket().expect("Failed to connect to the shire service socket");

                lock_block(&mut stream, name, lock.unwrap()).expect("Failed to lock block");
            }
            // I think that I should also just have a lock command?
            // Instead of making them start it with a lock
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
                // TODO: Implement service stop
            }
            ServiceAction::Restart => {
                println!("Restarting shire service...");
                // TODO: Implement service restart
            }
        },
    }
}
