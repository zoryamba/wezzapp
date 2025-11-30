use clap::Parser;
use tracing::info;
use tracing_subscriber::{EnvFilter, fmt};

use crate::cli::Command;
use crate::handlers::configure::ConfigureHandler;
use crate::handlers::get::GetHandler;
use crate::prompter::InquirePrompter;
use crate::store::TomlFileCredentialsStore;

mod cli;
mod handlers;
mod prompter;
mod store;

fn main() -> anyhow::Result<()> {
    init_tracing();

    let args = cli::Cli::parse();
    info!(?args, "Parsed CLI arguments");

    match args.command {
        Command::Configure { provider } => {
            ConfigureHandler::new(TomlFileCredentialsStore::new()?, InquirePrompter::new())
                .run(provider)
        }
        Command::Get {
            provider,
            address,
            date,
        } => GetHandler::run(provider, address, date),
    }
}

/// Initialize global tracing subscriber.
///
/// - Uses `RUST_LOG` if set (e.g. `RUST_LOG=wezzapp_cli=debug,wezzapp_core=trace`)
/// - Otherwise defaults to `info` for our crates.
fn init_tracing() {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("wezzapp_cli=info,wezzapp_core=info"));

    let _ = fmt()
        .with_env_filter(env_filter)
        .with_target(false)
        .compact()
        .try_init();
}
