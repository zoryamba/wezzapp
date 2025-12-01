use crate::apis::accu_weather::AccuWeatherClient;
use crate::apis::weather_api::WeatherApiClient;
use crate::credentials::Credentials;
use crate::provider::Provider;
use anyhow::{Result, anyhow};

mod accu_weather;
mod weather_api;

/// Result of a weather query, in a UI-friendly form.
#[derive(Debug)]
pub struct WeatherReport {
    pub provider: Provider,
    pub date: String,
    pub location: String,
    pub description: String,
    pub max_temperature: f64,
    pub min_temperature: f64,
}

/// abstraction over weather API client
pub trait ProviderClient {
    fn get_weather(&self, address: String, days: u32) -> Result<WeatherReport>;
}

/// Factory that returns a client for the given provider & credentials.
///
/// This is where you can hide the mapping:
///   Provider::WeatherApi   -> WeatherApiClient
///   Provider::AccuWeather  -> AccuWeatherClient
pub trait ProviderClientFactory {
    fn create_client(
        &self,
        provider: Provider,
        credentials: Credentials,
    ) -> Result<Box<dyn ProviderClient>>;
}

#[derive(Debug)]
pub struct HttpProviderClientFactory;

impl HttpProviderClientFactory {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for HttpProviderClientFactory {
    fn default() -> Self {
        HttpProviderClientFactory::new()
    }
}

impl ProviderClientFactory for HttpProviderClientFactory {
    fn create_client(
        &self,
        provider: Provider,
        credentials: Credentials,
    ) -> Result<Box<dyn ProviderClient>> {
        match (provider, credentials) {
            (Provider::WeatherApi, Credentials::WeatherApi { api_key }) => {
                Ok(Box::new(WeatherApiClient::new(api_key)))
            }
            (Provider::AccuWeather, Credentials::AccuWeather { api_key }) => {
                Ok(Box::new(AccuWeatherClient::new(api_key)))
            }
            _ => Err(anyhow!(
                "credentials type does not match provider: {provider:?}"
            )),
        }
    }
}
