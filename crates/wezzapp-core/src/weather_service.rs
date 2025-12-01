use crate::apis::{ProviderClientFactory, WeatherReport};
use crate::credentials::CredentialsStore;
use crate::provider::Provider;
use anyhow::{Context, Result, anyhow};
use chrono::{Local, NaiveDate};
use tracing::debug;

#[derive(Debug)]
pub struct WeatherService<S, F>
where
    S: CredentialsStore,
    F: ProviderClientFactory,
{
    store: S,
    factory: F,
}

impl<S, F> WeatherService<S, F>
where
    S: CredentialsStore,
    F: ProviderClientFactory,
{
    pub fn new(store: S, factory: F) -> Self {
        Self { store, factory }
    }

    /// Get weather for provided params
    pub fn get_weather(
        &mut self,
        address: String,
        date: Option<String>,
        provider: Option<Provider>,
    ) -> Result<WeatherReport> {
        debug!("Getting weather for address `{address}`");
        let days = if let Some(date) = date {
            days_from_today(&date)?
        } else {
            0
        };
        debug!("Days from today: {days}");

        let provider = self.resolve_provider(provider)?;

        let creds = self
            .store
            .get_credentials(provider)
            .context("failed to read credentials from store")?
            .ok_or_else(|| {
                anyhow!(
                    "No credentials found for provider `{provider:?}`. \
                     Please configure it first."
                )
            })?;
        debug!("Got credentials");

        let client = self.factory.create_client(provider, creds)?;

        client.get_weather(address, days)
    }

    fn resolve_provider(&mut self, provider: Option<Provider>) -> Result<Provider> {
        if let Some(p) = provider {
            return Ok(p);
        }

        self.store
            .get_default_provider()
            .context("failed to read default provider from store")?
            .ok_or_else(|| {
                anyhow!(
                    "No provider specified and no default provider set. \
                     Please configure a provider and/or set a default."
                )
            })
    }
}

pub fn days_from_today(date_str: &str) -> Result<u32> {
    debug!("Calculating days from today for date `{date_str}`");
    let target = NaiveDate::parse_from_str(&date_str, "%Y-%m-%d")
        .context("invalid date format (expected YYYY-MM-DD)")?;
    debug!("Parsed date `{date_str}` as `{target:?}`");

    let today = Local::now().date_naive();
    debug!("Today is `{today:?}`");

    if target < today {
        return Err(anyhow!("date is in the past"));
    }

    Ok((target - today).num_days() as u32)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration, Local, NaiveDate};

    fn fmt(d: NaiveDate) -> String {
        d.format("%Y-%m-%d").to_string()
    }

    #[test]
    fn today_returns_zero() {
        let today = Local::now().date_naive();
        let date_str = fmt(today);

        let result = days_from_today(&date_str).unwrap();
        assert_eq!(result, 0);
    }

    #[test]
    fn tomorrow_returns_one() {
        let today = Local::now().date_naive();
        let tomorrow = today + Duration::days(1);
        let date_str = fmt(tomorrow);

        let result = days_from_today(&date_str).unwrap();
        assert_eq!(result, 1);
    }

    #[test]
    fn past_date_returns_error() {
        let today = Local::now().date_naive();
        let past = today - Duration::days(1);
        let date_str = fmt(past);

        let err = days_from_today(&date_str).unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("in the past"),
            "unexpected error message: {msg}"
        );
    }

    #[test]
    fn invalid_format_returns_error() {
        let err = days_from_today("2025/01/01").unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("invalid date format"),
            "unexpected error message: {msg}"
        );
    }
}
