use anyhow::{Context, Result};
use inquire::{Confirm, Text};
use tracing::debug;
use wezzapp_core::credentials::Credentials;
use wezzapp_core::provider::Provider;

/// Trait for prompting user for input.
pub trait ConfigurePrompter {
    /// Ask user to confirm credentials overwrite.
    fn confirm_overwrite(&mut self, provider: Provider) -> Result<bool>;

    /// Ask user to confirm default provider change.
    fn confirm_set_default(&mut self, provider: Provider) -> Result<bool>;

    /// Ask user for credentials for a given provider.
    fn prompt_credentials(&mut self, provider: Provider) -> Result<Credentials>;
}

/// Real implementation using `inquire`.
pub struct InquirePrompter;

impl InquirePrompter {
    pub fn new() -> Self {
        Self
    }
}

impl ConfigurePrompter for InquirePrompter {
    fn confirm_overwrite(&mut self, _provider: Provider) -> Result<bool> {
        debug!("Confirming overwrite");
        let answer = Confirm::new("Credentials already exist. Overwrite?")
            .with_default(true)
            .prompt()
            .context("failed to read confirmation from stdin")?;

        Ok(answer)
    }

    fn confirm_set_default(&mut self, _provider: Provider) -> Result<bool> {
        debug!("Confirming default provider change");
        let answer = Confirm::new("Do you want to make this provider the default?")
            .with_default(true)
            .prompt()
            .context("failed to read confirmation from stdin")?;

        Ok(answer)
    }

    fn prompt_credentials(&mut self, provider: Provider) -> Result<Credentials> {
        debug!("Prompting for credentials for provider {:?}", provider);
        match provider {
            Provider::WeatherApi => {
                let api_key = Text::new("Enter WeatherAPI API key:")
                    .with_help_message("Sign up at https://www.weatherapi.com/")
                    .prompt()
                    .context("failed to read WeatherAPI API key from stdin")?;

                Ok(Credentials::WeatherApi { api_key })
            }

            Provider::AccuWeather => {
                let api_key = Text::new("Enter AccuWeather API key:")
                    .with_help_message("Visit https://developer.accuweather.com/")
                    .prompt()
                    .context("failed to read AccuWeather API key from stdin")?;

                Ok(Credentials::AccuWeather { api_key })
            }
        }
    }
}
