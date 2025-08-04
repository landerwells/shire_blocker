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
    /// Install service dependencies (e.g. plist)
    Install,
}

fn main() {
    let args = Args::parse();

    // let service = launchctl::Service::builder()
    //     .name("com.sylvanfranklin.srhd")
    //     .build();
    // srhd::service::install(&service).unwrap();

    let socket_path = "/tmp/shire_cli.sock";

    let mut cli_sock = match UnixStream::connect(socket_path) {
        Ok(sock) => sock,
        Err(e) => {
            eprintln!(
                "Failed to connect to Shire daemon at '{socket_path}': {e}.\n\
                 Is the daemon running? Try starting it with `shire service start`");
            return;
        }
    };

    match args.command {
        Commands::Block { action } => match action {
            BlockAction::List => {
                // TODO: Implement block listing
                // println!("Listing all available blocks...");
                // I want the printing to be similar to something like gh repo list
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
                let _ = service::install();
                // TODO: Implement service start
            }
            ServiceAction::Stop => {
                println!("Stopping shire service...");
                // TODO: Implement service stop
            }
            ServiceAction::Restart => {
                println!("Restarting shire service...");
                // TODO: Implement service restart
            }
            ServiceAction::Install => {
                println!("Installing service dependencies...");
                // TODO: Implement service install
            }
        },
    }
}
