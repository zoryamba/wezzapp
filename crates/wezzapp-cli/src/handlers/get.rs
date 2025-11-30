use crate::cli::ProviderCli;
use crate::store::TomlFileCredentialsStore;
use anyhow::Result;
use wezzapp_core::apis::{HttpProviderClientFactory, WeatherReport};
use wezzapp_core::weather_service::WeatherService;

pub struct GetHandler {
    service: WeatherService<TomlFileCredentialsStore, HttpProviderClientFactory>,
}

impl GetHandler {
    pub fn new(
        service: WeatherService<TomlFileCredentialsStore, HttpProviderClientFactory>,
    ) -> Self {
        Self { service }
    }

    /// Run the `get` flow.
    ///
    /// - Resolve provider: CLI override or default from store.
    /// - Load credentials for that provider.
    /// - Create provider client from factory.
    /// - Fetch weather and print human-readable output.
    pub fn run(
        &mut self,
        address: String,
        date: Option<String>,
        provider: Option<ProviderCli>,
    ) -> Result<()> {
        let report = self.service.get_weather(address, date, provider.map(Into::into))?;

        self.render_report(report);

        Ok(())
    }

    /// Renders weather report
    /// Can be moved to separate render layer if needed
    fn render_report(&mut self, report: WeatherReport) {
        println!("{:?}", report);
    }
}
