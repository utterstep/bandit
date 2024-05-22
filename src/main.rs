use clap::Parser;
use eyre::Result;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Registry};
use tracing_tree::HierarchicalLayer;

mod cli;
mod client;
mod consts;
mod server;

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    // enable tracing
    Registry::default()
        .with(EnvFilter::from_default_env())
        .with(
            HierarchicalLayer::new(4)
                .with_targets(true)
                .with_indent_lines(true)
                .with_bracketed_fields(true)
                .with_thread_names(false)
                .with_thread_ids(true),
        )
        .init();

    let cli = cli::Cli::parse();
    let packet_size = cli.packet_size.0 as usize;

    match cli.command {
        cli::Command::Server {
            bind_to,
            payload_size,
        } => {
            server::server(bind_to, payload_size.0 as usize, packet_size).await?;
        }
        cli::Command::Client { server } => client::client_tcp(&server, packet_size).await?,
    }

    Ok(())
}
