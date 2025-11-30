use clap::{Parser, Subcommand, ValueEnum};
use wezzapp_core::provider::Provider;

/// Top-level CLI for the `wezzapp` command.
///
/// Examples:
///   wezzapp configure weatherapi
///   wezzapp get "Kyiv, Ukraine"
///   wezzapp get "Kyiv, Ukraine" "2024-11-29"
///   wezzapp get "Kyiv, Ukraine" "2024-11-29" --provider accuweather
#[derive(Debug, Parser)]
#[command(
    name = "wezzapp",
    version,
    about = "A simple multi-provider weather CLI",
    author = "zoryamba"
)]
pub struct Cli {
    /// Top-level command.
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Configure credentials for a specific weather provider.
    ///
    /// Interactively prompts user for credentials. Allows to update default provider
    Configure {
        /// Weather provider to configure credentials for.
        #[arg(value_enum)]
        provider: ProviderCli,
    },

    /// Get weather for a given address (and optional date).
    ///
    /// If date is omitted, "now" is used.
    Get {
        /// Address/location string, e.g. "Kyiv, Ukraine"
        address: String,

        /// Optional date, e.g. "2024-11-29". If not provided, we treat it as "now".
        date: Option<String>,

        /// Optional provider override. If omitted, user's default is used.
        #[arg(long, value_enum)]
        provider: Option<ProviderCli>,
    },
}

/// Supported weather providers.
///
/// Right now we only support:
/// - WeatherApi
/// - AccuWeather
#[derive(Debug, Copy, Clone, Eq, PartialEq, ValueEnum)]
pub enum ProviderCli {
    /// https://www.weatherapi.com/
    #[value(name = "weatherapi")]
    WeatherApi,

    /// https://developer.accuweather.com/
    #[value(name = "accuweather")]
    AccuWeather,
}

impl From<Provider> for ProviderCli {
    fn from(provider: Provider) -> Self {
        match provider {
            Provider::WeatherApi => Self::WeatherApi,
            Provider::AccuWeather => Self::AccuWeather,
        }
    }
}

impl From<ProviderCli> for Provider {
    fn from(provider: ProviderCli) -> Self {
        match provider {
            ProviderCli::WeatherApi => Self::WeatherApi,
            ProviderCli::AccuWeather => Self::AccuWeather,
        }
    }
}