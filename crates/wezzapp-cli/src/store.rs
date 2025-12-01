use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use tracing::debug;
use wezzapp_core::credentials::{Credentials, CredentialsStore};
use wezzapp_core::provider::Provider;

/// On-disk configuration format for credentials & default provider.
///
/// Example TOML:
/// ```toml
/// default = "weatherapi"
///
/// [providers.accuweather.accuweather]
/// api_key = "abc"
///
/// [providers.weatherapi.weatherapi]
/// api_key = "xyz"
/// ```
#[derive(Default, Serialize, Deserialize)]
struct Config {
    /// Default provider (string encoded via `Provider` serde rename).
    #[serde(default)]
    default: Option<Provider>,

    /// Map from provider key ("weatherapi", "accuweather") to credentials.
    #[serde(default)]
    providers: HashMap<Provider, Credentials>,
}

/// TOML-file-based implementation of `CredentialsStore`.
///
/// Stored in:
///   `<home>/.wezzapp/credentials.toml`
pub struct TomlFileCredentialsStore {
    path: std::path::PathBuf,
    config: Config,
}

impl TomlFileCredentialsStore {
    pub fn new() -> Result<Self> {
        debug!("Creating new TomlFileCredentialsStore");
        let dirs =
            directories::UserDirs::new().context("failed to determine user home directory")?;
        let home = dirs.home_dir();
        let dir = home.join(".wezzapp");
        let path = dir.join("credentials.toml");
        debug!("Using credentials file at {}", path.display());

        Self::new_with_path(&path)
    }

    fn new_with_path(path: &Path) -> Result<Self> {
        debug!(
            "Creating new TomlFileCredentialsStore with path {}",
            path.display()
        );
        let config = if path.exists() {
            let contents = fs::read_to_string(path)
                .context(format!("failed to read config file {}", path.display()))?;
            debug!("Loaded credentials from {}", path.display());

            toml::from_str(&contents).context("failed to parse credentials TOML")?
        } else {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)
                    .context(format!("failed to create directory {}", parent.display()))?;
                debug!(
                    "Created directory {} for credentials file",
                    parent.display()
                );
            }
            Config::default()
        };
        debug!("Config created");

        Ok(Self {
            path: path.to_path_buf(),
            config,
        })
    }

    fn save_file(&self) -> Result<()> {
        debug!("Saving credentials to {}", self.path.display());
        let tmp = self.path.with_extension("tmp");

        let data =
            toml::to_string_pretty(&self.config).context("failed to serialize credentials TOML")?;

        fs::write(&tmp, data).context(format!("failed to write config file {}", tmp.display()))?;
        debug!("Wrote credentials to {}", tmp.display());

        fs::rename(&tmp, &self.path).context(format!(
            "failed to rename tmp config file {}",
            tmp.display()
        ))?;
        debug!("Renamed tmp file to {}", self.path.display());

        Ok(())
    }
}

impl CredentialsStore for TomlFileCredentialsStore {
    fn set_credentials(&mut self, provider: Provider, credentials: &Credentials) -> Result<()> {
        debug!("Setting credentials for provider {:?}", provider);
        self.config.providers.insert(provider, credentials.clone());
        self.save_file().context("failed to save credentials")
    }

    fn get_credentials(&self, provider: Provider) -> Result<Option<Credentials>> {
        debug!("Getting credentials for provider {:?}", provider);
        Ok(self.config.providers.get(&provider).cloned())
    }

    fn set_default_provider(&mut self, provider: Provider) -> Result<()> {
        debug!("Setting default provider to {:?}", provider);
        self.config.default = Some(provider);
        self.save_file()
    }

    fn get_default_provider(&self) -> Result<Option<Provider>> {
        debug!("Getting default provider");
        Ok(self.config.default)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;
    use tempfile::TempDir;

    /// Helper struct to keep the temp dir alive while we work with the store.
    struct StoreFixture {
        _tmpdir: TempDir,
        store: TomlFileCredentialsStore,
    }

    impl StoreFixture {
        fn new() -> Self {
            let tmpdir = tempfile::tempdir().expect("create temp dir");
            let path = tmpdir.path().join("credentials.toml");

            let store =
                TomlFileCredentialsStore::new_with_path(&path).expect("create file-based store");

            StoreFixture {
                _tmpdir: tmpdir,
                store,
            }
        }

        /// Create a second store reading from the same path to test persistence.
        fn reopen(&self) -> TomlFileCredentialsStore {
            TomlFileCredentialsStore::new_with_path(&self.store.path)
                .expect("reopen file-based store")
        }
    }

    #[test]
    fn new_creates_empty_config_if_file_missing() {
        let fixture = StoreFixture::new();

        assert!(
            fixture.store.config.providers.is_empty(),
            "providers map should be empty"
        );
        assert!(
            fixture.store.config.default.is_none(),
            "default provider should be None"
        );

        assert!(
            !fixture.store.path.exists(),
            "credentials file should not exist before first save"
        );
    }

    #[rstest]
    #[case(
        Provider::WeatherApi,
        Credentials::WeatherApi { api_key: "weather-key".into() }
    )]
    #[case(
        Provider::AccuWeather,
        Credentials::AccuWeather { api_key: "accu-key".into() }
    )]
    fn set_and_get_credentials_roundtrip(#[case] provider: Provider, #[case] creds: Credentials) {
        let mut fixture = StoreFixture::new();

        fixture
            .store
            .set_credentials(provider, &creds)
            .expect("set_credentials");

        assert!(
            fixture.store.path.exists(),
            "credentials file should be created on first save"
        );

        let loaded = fixture
            .store
            .get_credentials(provider)
            .expect("get_credentials");

        assert!(
            Some(creds) == loaded,
            "stored credentials should match what we get back"
        );
    }

    #[test]
    fn set_default_provider_and_get_default_credentials() {
        let mut fixture = StoreFixture::new();

        fixture
            .store
            .set_default_provider(Provider::AccuWeather)
            .expect("set_default_provider");

        let default = fixture
            .store
            .get_default_provider()
            .expect("get_default_provider");

        assert_eq!(
            Some(Provider::AccuWeather),
            default,
            "default credentials should come from AccuWeather"
        );

        fixture
            .store
            .set_default_provider(Provider::WeatherApi)
            .expect("set_default_provider");

        let default2 = fixture
            .store
            .get_default_provider()
            .expect("get_default_provider");

        assert_eq!(
            Some(Provider::WeatherApi),
            default2,
            "default credentials should now come from WeatherAPI"
        );
    }

    #[test]
    fn credentials_persist_across_reloads() {
        let mut fixture = StoreFixture::new();

        let creds = Credentials::WeatherApi {
            api_key: "persisted-key".into(),
        };

        fixture
            .store
            .set_credentials(Provider::WeatherApi, &creds)
            .expect("set_credentials");
        fixture
            .store
            .set_default_provider(Provider::WeatherApi)
            .expect("set_default_provider");

        let store2 = fixture.reopen();

        let loaded_creds = store2
            .get_credentials(Provider::WeatherApi)
            .expect("get_credentials");
        let default_provider = store2.get_default_provider().expect("get_default_provider");

        assert!(
            Some(creds) == loaded_creds,
            "credentials should survive reload"
        );
        assert_eq!(
            Some(Provider::WeatherApi),
            default_provider,
            "default credentials should survive reload"
        );
    }
}
