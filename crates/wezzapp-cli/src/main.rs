use clap::Parser;
use tracing::info;
use tracing_subscriber::{EnvFilter, fmt};
use wezzapp_core::apis::HttpProviderClientFactory;
use wezzapp_core::weather_service::WeatherService;
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

    match args.command {
        Command::Configure { provider } => {
            ConfigureHandler::new(TomlFileCredentialsStore::new()?, InquirePrompter::new())
                .run(provider)
        }
        Command::Get {
            address,
            date,
            provider,
        } => {
            let store = TomlFileCredentialsStore::new()?;

            let factory = HttpProviderClientFactory::new(); // your implementation

            let service = WeatherService::new(store, factory);
            let mut handler = GetHandler::new(service);


            handler.run(address, date, provider)
        },
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
