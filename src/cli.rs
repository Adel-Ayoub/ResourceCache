use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(arg_required_else_help(true),author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: SubCommand,

    #[arg(short, long, required=false, help = "location of the config file, default: $HOME/.config/resource-cache/config.conf")]
    pub config: Option<PathBuf>,
}

#[derive(Subcommand, Debug)]
pub enum SubCommand {
    #[command(arg_required_else_help(true))]
    Get {
        #[arg(short = 'o', long = "output", required = false)]
        filename: String,

        url: String,
        #[arg(short, long, required = false)]
        refresh: bool,
        #[arg(short, long, required = false, help="time offset in seconds when the file will expire")]
        expire_time: Option<u64>,
    },
    Stats,
    #[command(about = "remove expired files")]
    CleanExpired,
    #[command(about = "delete all the files in the cache")]
    Delete,
}
