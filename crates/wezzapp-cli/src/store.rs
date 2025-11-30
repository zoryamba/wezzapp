use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
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
///
/// Provider keys may seem repetitive, but this structure may be useful if in future we decide to store multiple
/// credentials for the same customer under some custom alias (I would definitely do, if I had free time :))
#[derive(Debug, Default, Serialize, Deserialize)]
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
        let dirs =
            directories::UserDirs::new().context("failed to determine user home directory")?;
        let home = dirs.home_dir();
        let dir = home.join(".wezzapp");
        let path = dir.join("credentials.toml");

        Self::new_with_path(&path)
    }

    fn new_with_path(path: &Path) -> Result<Self> {
        let config = if path.exists() {
            let contents = fs::read_to_string(&path)
                .context(format!("failed to read config file {}", path.display()))?;

            toml::from_str(&contents).context("failed to parse credentials TOML")?
        } else {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)
                    .context(format!("failed to create directory {}", parent.display()))?;
            }
            Config::default()
        };

        Ok(Self {
            path: path.to_path_buf(),
            config,
        })
    }

    fn save_file(&self) -> Result<()> {
        let tmp = self.path.with_extension("tmp");

        let data =
            toml::to_string_pretty(&self.config).context("failed to serialize credentials TOML")?;
        fs::write(&tmp, data).context(format!("failed to write config file {}", tmp.display()))?;
        fs::rename(&tmp, &self.path).context(format!(
            "failed to rename tmp config file {}",
            tmp.display()
        ))?;

        Ok(())
    }
}

impl CredentialsStore for TomlFileCredentialsStore {
    fn set_credentials(&mut self, provider: Provider, credentials: &Credentials) -> Result<()> {
        self.config.providers.insert(provider, credentials.clone());
        self.save_file().context("failed to save credentials")
    }

    fn get_credentials(&self, provider: Provider) -> Result<Option<Credentials>> {
        Ok(self.config.providers.get(&provider).cloned())
    }

    fn set_default_provider(&mut self, provider: Provider) -> Result<()> {
        self.config.default = Some(provider);
        self.save_file()
    }

    fn get_default_provider(&self) -> Result<Option<Provider>> {
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

        assert_eq!(
            Some(creds),
            loaded,
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

        assert_eq!(
            Some(creds.clone()),
            loaded_creds,
            "credentials should survive reload"
        );
        assert_eq!(
            Some(Provider::WeatherApi),
            default_provider,
            "default credentials should survive reload"
        );
    }
}
