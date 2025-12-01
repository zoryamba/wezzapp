use crate::apis::{ProviderClient, WeatherReport};
use crate::provider::Provider;
use anyhow::{Context, Result, anyhow};
use chrono::{DateTime, FixedOffset, NaiveDate};
use reqwest::Url;
use reqwest::blocking::Client;
use reqwest::header::AUTHORIZATION;
use serde::{Deserialize, Deserializer, de};
use tracing::debug;

/// Http client for AccuWeather API
#[derive(Debug)]
pub struct AccuWeatherClient<'a> {
    api_key: String,
    url: &'a str,
    client: Client,
}

impl AccuWeatherClient<'static> {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            client: Client::new(),
            url: "https://dataservice.accuweather.com/",
        }
    }
}

impl AccuWeatherClient<'static> {
    fn get_location_key(&self, address: String) -> Result<AccuWeatherLocationResponse> {
        debug!("Getting location key for address `{address}`");
        let mut url = Url::parse(self.url).context("Error parsing AccuWeather API URL")?;
        url = url
            .join("locations/v1/search")
            .context("Error joining AccuWeather API URL")?;
        {
            let mut qp = url.query_pairs_mut();
            qp.append_pair("q", &address);
        }
        debug!("AccuWeather API URL: {url:?}");

        let resp = self
            .client
            .get(url)
            .header(AUTHORIZATION, format!("Bearer {}", self.api_key))
            .send()
            .context("failed to send request to AccuWeather API")?
            .error_for_status()
            .context("AccuWeather API returned error status")?;
        debug!("AccuWeather API response: {resp:?}");

        let mut body: Vec<AccuWeatherLocationResponse> = resp
            .json()
            .context("failed to deserialize AccuWeather API JSON")?;
        debug!("AccuWeather API body: {body:?}");

        let location_key = body
            .pop()
            .context("Address not found, please, use more accurate address, eg: Kyiv, Ukraine")?;
        debug!("AccuWeather API location key: {location_key:?}");

        Ok(location_key)
    }
}

impl ProviderClient for AccuWeatherClient<'static> {
    fn get_weather(&self, address: String, day_from_today: u32) -> Result<WeatherReport> {
        debug!("Getting weather for address `{address} day from today: {day_from_today}`");
        let days = day_from_today + 1;
        // It only supports up to 5 days on the free plan.
        if days > 5 {
            return Err(anyhow!(
                "AccuWeather API only supports up to 5 days forecast (including today)."
            ));
        }

        let location = self.get_location_key(address)?;

        let mut url = Url::parse(self.url).context("Error parsing AccuWeather API URL")?;
        url = url
            .join(&format!("forecasts/v1/daily/5day/{}", location.key))
            .context("Error joining AccuWeather API URL")?;
        {
            let mut qp = url.query_pairs_mut();
            qp.append_pair("metric", &true.to_string());
        }
        debug!("AccuWeather API URL: {url:?}");

        let resp = self
            .client
            .get(url)
            .header(AUTHORIZATION, format!("Bearer {}", self.api_key))
            .send()
            .context("failed to send request to AccuWeather API")?
            .error_for_status()
            .context("AccuWeather API returned error status")?;
        debug!("AccuWeather API response: {resp:?}");

        let body: AccuWeatherForecastResponse = resp
            .json()
            .context("Failed to deserialize AccuWeather API JSON")?;
        debug!("AccuWeather API body: {body:?}");

        let forecast = body
            .daily_forecasts
            .get(day_from_today as usize)
            .context("Wrong number of days in API response")?;
        debug!("AccuWeather API forecast: {forecast:?}");

        Ok(WeatherReport {
            provider: Provider::WeatherApi,
            date: forecast.date.clone().to_string(),
            location: format!(
                "{}, {}",
                location.localized_name, location.country.localized_name
            ),
            description: format!(
                "Day: {}, Night: {}",
                forecast.day.icon_prase, forecast.night.icon_prase
            ),
            max_temperature: forecast.temperature.minimum.value,
            min_temperature: forecast.temperature.maximum.value,
        })
    }
}

#[derive(Debug, Deserialize)]
struct AccuWeatherLocationResponse {
    #[serde(rename = "Key")]
    key: String,
    #[serde(rename = "LocalizedName")]
    localized_name: String,
    #[serde(rename = "Country")]
    country: AccuWeatherCountryResponse,
}
#[derive(Debug, Deserialize)]
struct AccuWeatherCountryResponse {
    #[serde(rename = "LocalizedName")]
    localized_name: String,
}

#[derive(Debug, Deserialize)]
struct AccuWeatherForecastResponse {
    #[serde(rename = "DailyForecasts")]
    daily_forecasts: Vec<AccuWeatherDailyForecastResponse>,
}

#[derive(Debug, Deserialize)]
struct AccuWeatherDailyForecastResponse {
    #[serde(rename = "Date", deserialize_with = "deserialize_naive_date_from_rfc")]
    date: NaiveDate,
    #[serde(rename = "Temperature")]
    temperature: AccuWeatherTemperatureResponse,
    #[serde(rename = "Day")]
    day: AccuWeatherDayNightResponse,
    #[serde(rename = "Night")]
    night: AccuWeatherDayNightResponse,
}

#[derive(Debug, Deserialize)]
struct AccuWeatherTemperatureResponse {
    #[serde(rename = "Minimum")]
    minimum: AccuWeatherTemperatureValueResponse,
    #[serde(rename = "Maximum")]
    maximum: AccuWeatherTemperatureValueResponse,
}

#[derive(Debug, Deserialize)]
struct AccuWeatherTemperatureValueResponse {
    #[serde(rename = "Value")]
    value: f64,
}

#[derive(Debug, Deserialize)]
struct AccuWeatherDayNightResponse {
    #[serde(rename = "IconPhrase")]
    icon_prase: String,
}

fn deserialize_naive_date_from_rfc<'de, D>(deserializer: D) -> Result<NaiveDate, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    let datetime_with_offset: DateTime<FixedOffset> =
        DateTime::parse_from_rfc3339(&s).map_err(de::Error::custom)?;

    Ok(datetime_with_offset.date_naive())
}
