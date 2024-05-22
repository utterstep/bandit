use bytesize::ByteSize;
use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "bandit")]
#[command(about = "Bandwidth estimation tool")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
    #[arg(short, long, default_value = "32664")]
    pub packet_size: ByteSize,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    Server {
        #[arg(short, long)]
        bind_to: String,
        #[arg(short, long)]
        payload_size: ByteSize,
    },
    Client {
        #[arg(short, long)]
        server: String,
    },
}
