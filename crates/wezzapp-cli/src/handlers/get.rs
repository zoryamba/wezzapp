use crate::cli::ProviderCli;

pub struct GetHandler;

/// `get` command handler.
impl GetHandler {
    pub fn run(
        _provider: Option<ProviderCli>,
        address: String,
        date: Option<String>,
    ) -> anyhow::Result<()> {
        let when = date.as_deref().unwrap_or("now");
        // Later:
        // - determine active provider from config
        // - call core API to fetch weather
        // - format and display result
        println!("(stub) get weather for `{address}` at `{when}`");
        Ok(())
    }
}
