mod commands;
mod config;
mod daemon;
mod service;
mod state;

use crate::daemon::start_daemon;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "shire",
    version = "0.0",
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
    // Schedule {
    //     #[command(subcommand)]
    //     action: ScheduleAction,
    // },
    /// Manage the shire service
    Service {
        #[command(subcommand)]
        action: ServiceAction,
    },
    /// Launch the daemon
    Daemon {
        #[arg(long)]
        config: Option<String>, // e.g. duration
    },
}

#[derive(Subcommand)]
enum BlockAction {
    /// List all available blocks
    List,
    /// Start a block with or without a lock
    Start {
        name: String,
        #[arg(long)]
        // Need to make this more obvious to users
        lock: Option<String>, // e.g. duration
    },
    /// Stop a block
    Stop { name: String },
}

// #[derive(Subcommand)]
// enum ScheduleAction {
//     /// List the schedule
//     List,
// }

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

fn main() {
    let args = Args::parse();

    match args.command {
        Commands::Block { action } => match action {
            BlockAction::List => {
                let _ = commands::list_blocks();
            }
            BlockAction::Start { name, lock } => {
                commands::start_block(&name, lock.as_deref());
            }
            BlockAction::Stop { name } => {
                commands::stop_block(&name);
            }
        },
        // Commands::Schedule { action } => match action {
        //     ScheduleAction::List => {
        //         println!("Listing the block schedule...");
        //     }
        // },
        // TODO: Better error handling for these events
        Commands::Service { action } => match action {
            ServiceAction::Start => {
                let _ = service::start();
            }
            ServiceAction::Stop => {
                let _ = service::stop();
            }
            ServiceAction::Restart => {
                service::restart();
            }
            ServiceAction::Uninstall => {
                let _ = service::uninstall();
            }
        },
        Commands::Daemon { config } => {
            start_daemon(config);
        }
    }
}
