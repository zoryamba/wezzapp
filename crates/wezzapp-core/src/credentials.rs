use crate::provider::Provider;
use serde::{Deserialize, Serialize};

/// Credentials for a concrete provider.
/// Use enum, since each provider may have different auth fields
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Credentials {
    WeatherApi { api_key: String },
    AccuWeather { api_key: String },
}

impl Credentials {
    /// Return which provider these credentials belong to.
    pub fn provider(&self) -> Provider {
        match self {
            Credentials::WeatherApi { .. } => Provider::WeatherApi,
            Credentials::AccuWeather { .. } => Provider::AccuWeather,
        }
    }
}

/// Abstraction over a storage for credentials and default provider.
///
/// Different frontends (CLI, GUI, etc.) can have their own implementations:
/// - TOML file
/// - OS keychain
/// - encrypted DB
pub trait CredentialsStore {
    /// Set credentials for the given provider.
    fn set_credentials(
        &mut self,
        provider: Provider,
        credentials: &Credentials,
    ) -> anyhow::Result<()>;

    /// Get credentials for the given provider.
    fn get_credentials(&self, provider: Provider) -> anyhow::Result<Option<Credentials>>;

    /// Set the default provider to use when user does not specify it explicitly.
    fn set_default_provider(&mut self, provider: Provider) -> anyhow::Result<()>;

    /// Get the default provider, if configured.
    fn get_default_provider(&self) -> anyhow::Result<Option<Provider>>;
}
