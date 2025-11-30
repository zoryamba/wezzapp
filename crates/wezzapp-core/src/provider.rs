use serde::{Deserialize, Serialize};

/// Supported weather providers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Provider {
    WeatherApi,
    AccuWeather,
}

impl Provider {
    /// Get base URL for the provider.
    pub fn get_url(&self) -> &'static str {
        match self {
            Provider::WeatherApi => "https://api.openweathermap.org/data/2.5/",
            Provider::AccuWeather => "https://dataservice.accuweather.com/",
        }
    }
}
