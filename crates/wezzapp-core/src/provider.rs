use serde::{Deserialize, Serialize};

/// Supported weather providers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Provider {
    WeatherApi,
    AccuWeather,
}
