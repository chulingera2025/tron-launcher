mod cli;
mod commands;
mod constants;
mod core;
mod error;
mod models;
mod utils;

use clap::Parser;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "tronctl=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let cli = cli::Cli::parse();

    let result = match cli.command {
        cli::Commands::Init {
            snapshot,
            version,
            skip_checks,
        } => commands::init::execute(snapshot, version, skip_checks).await,

        cli::Commands::Start { daemon } => commands::start::execute(daemon).await,

        cli::Commands::Stop { force } => {
            commands::stop::execute(force)?;
            Ok(())
        }

        cli::Commands::Restart { daemon } => commands::restart::execute(daemon).await,

        cli::Commands::Status { verbose } => commands::status::execute(verbose).await,

        cli::Commands::Logs { follow, lines } => commands::logs::execute(follow, lines).await,

        cli::Commands::Clean { yes } => commands::clean::execute(yes).await,

        cli::Commands::Systemd { force } => commands::systemd::execute(force).await,
    };

    if let Err(e) = result {
        eprintln!("错误: {}", e);
        std::process::exit(1);
    }

    Ok(())
}
