use crate::apis::{ProviderClient, WeatherReport};
use crate::provider::Provider;
use anyhow::{Context, anyhow};
use reqwest::Url;
use reqwest::blocking::Client;
use serde::Deserialize;
use tracing::debug;

/// Http client for WeatherAPI
#[derive(Debug)]
pub struct WeatherApiClient<'a> {
    api_key: String,
    url: &'a str,
    client: Client,
}

impl WeatherApiClient<'static> {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            client: Client::new(),
            url: "https://api.weatherapi.com/v1/",
        }
    }
}

impl ProviderClient for WeatherApiClient<'static> {
    fn get_weather(&self, address: String, day_from_today: u32) -> anyhow::Result<WeatherReport> {
        debug!("Getting weather for address `{address} day from today: {day_from_today}`");
        let days = day_from_today + 1;

        if days > 14 {
            return Err(anyhow!(
                "WeatherAPI only supports up to 14 days forecast (including today)."
            ));
        }

        let mut url = Url::parse(self.url).context("Error parsing WeatherAPI URL")?;
        url = url.join("forecast.json").context("Error joining WeatherAPI URL")?;
        {
            let mut qp = url.query_pairs_mut();
            qp.append_pair("key", &self.api_key);
            qp.append_pair("q", &address);
            qp.append_pair("days", &(days).to_string());
        }
        debug!("WeatherAPI URL: {url:?}");

        let resp = self
            .client
            .get(url)
            .send()
            .context("failed to send request to WeatherAPI")?
            .error_for_status()
            .context("WeatherAPI returned error status")?;
        debug!("WeatherAPI response: {resp:?}");

        let body: WeatherApiResponse = resp
            .json()
            .context("failed to deserialize WeatherAPI JSON")?;
        debug!("WeatherAPI body: {body:?}");

        let forecast = body
            .forecast
            .forecastday
            .get(day_from_today as usize)
            .context("wrong number of days in API response")?;
        debug!("WeatherAPI forecast: {forecast:?}");

        Ok(WeatherReport {
            provider: Provider::WeatherApi,
            date: forecast.date.clone(),
            location: format!("{}, {}", body.location.name, body.location.country),
            description: forecast.day.condition.text.clone(),
            max_temperature: forecast.day.maxtemp_c,
            min_temperature: forecast.day.mintemp_c,
        })
    }
}

#[derive(Debug, Deserialize)]
struct WeatherApiResponse {
    location: WeatherApiLocation,
    forecast: WeatherApiForecast,
}

#[derive(Debug, Deserialize)]
struct WeatherApiForecast {
    forecastday: Vec<WeatherApiForecastDay>,
}

#[derive(Debug, Deserialize)]
struct WeatherApiLocation {
    name: String,
    country: String,
}

#[derive(Debug, Deserialize)]
struct WeatherApiForecastDay {
    date: String,
    day: WeatherApiDay,
}

#[derive(Debug, Deserialize)]
struct WeatherApiDay {
    maxtemp_c: f64,
    mintemp_c: f64,
    condition: WeatherApiCondition,
}

#[derive(Debug, Deserialize)]
struct WeatherApiCondition {
    text: String,
}
