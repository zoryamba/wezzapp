use crate::cli::ProviderCli;
use crate::prompter::ConfigurePrompter;
use anyhow::{Context, Result};
use tracing::debug;
use wezzapp_core::credentials::CredentialsStore;
use wezzapp_core::provider::Provider;

/// `configure` command handler.
pub struct ConfigureHandler<S, P>
where
    S: CredentialsStore,
    P: ConfigurePrompter,
{
    store: S,
    prompter: P,
}

impl<S, P> ConfigureHandler<S, P>
where
    S: CredentialsStore,
    P: ConfigurePrompter,
{
    pub fn new(store: S, prompter: P) -> Self {
        Self { store, prompter }
    }
    pub fn run(&mut self, provider_cli: ProviderCli) -> Result<()> {
        let provider: Provider = provider_cli.into();
        debug!("Configuring provider: {:?}", provider);

        let existing = self.store.get_credentials(provider)?;
        debug!("Existing credentials {}", existing.is_some());

        let overwrite = if existing.is_some() {
            self.prompter.confirm_overwrite(provider)?
        } else {
            true
        };
        debug!("Overwrite credentials: {:?}", overwrite);

        if overwrite {
            let new_credentials = self.prompter.prompt_credentials(provider)?;

            self.store
                .set_credentials(provider, &new_credentials)
                .context("failed to save credentials")?;

            println!("Credentials for `{provider_cli}` were saved.");
        };

        let current_default = self.store.get_default_provider()?;
        debug!("Current default provider: {:?}", current_default);

        let set_default = match current_default {
            None => true,
            Some(default) if default == provider => false,
            Some(_) => self.prompter.confirm_set_default(provider)?,
        };
        debug!("Set default provider: {:?}", set_default);

        if set_default {
            self.store
                .set_default_provider(provider)
                .context("failed to set default provider")?;

            println!("Provider `{provider_cli}` was set as default.");
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use wezzapp_core::credentials::Credentials;

    /// In-memory implementation of CredentialsStore for tests.
    #[derive(Default)]
    struct InMemoryStore {
        default: Option<Provider>,
        providers: HashMap<Provider, Credentials>,
    }

    impl CredentialsStore for &mut InMemoryStore {
        fn set_credentials(&mut self, provider: Provider, credentials: &Credentials) -> Result<()> {
            self.providers.insert(provider, credentials.clone());
            Ok(())
        }

        fn get_credentials(&self, provider: Provider) -> Result<Option<Credentials>> {
            Ok(self.providers.get(&provider).cloned())
        }

        fn set_default_provider(&mut self, provider: Provider) -> Result<()> {
            self.default = Some(provider);
            Ok(())
        }

        fn get_default_provider(&self) -> Result<Option<Provider>> {
            Ok(self.default)
        }
    }

    /// Mock prompter that lets tests control answers.
    struct MockPrompter {
        pub overwrite_answer: bool,
        pub set_default_answer: bool,
        pub credentials_to_return: Credentials,

        pub overwrite_called: bool,
        pub set_default_called: bool,
        pub credentials_prompt_called: bool,
    }

    impl ConfigurePrompter for &mut MockPrompter {
        fn confirm_overwrite(&mut self, _provider: Provider) -> Result<bool> {
            self.overwrite_called = true;
            Ok(self.overwrite_answer)
        }

        fn confirm_set_default(&mut self, _provider: Provider) -> Result<bool> {
            self.set_default_called = true;
            Ok(self.set_default_answer)
        }

        fn prompt_credentials(&mut self, _provider: Provider) -> Result<Credentials> {
            self.credentials_prompt_called = true;
            Ok(self.credentials_to_return.clone())
        }
    }

    fn sample_weatherapi_creds() -> Credentials {
        Credentials::WeatherApi {
            api_key: "TEST_KEY".to_string(),
        }
    }

    #[test]
    fn configure_new_provider_with_no_default_sets_creds_and_default() {
        let provider = ProviderCli::WeatherApi;

        let mut store = InMemoryStore::default();
        let mut prompter = MockPrompter {
            overwrite_answer: true,
            set_default_answer: true,
            credentials_to_return: sample_weatherapi_creds(),
            overwrite_called: false,
            set_default_called: false,
            credentials_prompt_called: false,
        };

        ConfigureHandler::new(&mut store, &mut prompter)
            .run(provider)
            .expect("configuration should succeed");

        let saved = store
            .providers
            .get(&provider.into())
            .cloned()
            .expect("credentials must be present");

        assert!(
            saved
                == Credentials::WeatherApi {
                    api_key: "TEST_KEY".to_string()
                }
        );
        assert_eq!(store.default, Some(provider.into()));
        assert!(!prompter.overwrite_called);
        assert!(prompter.credentials_prompt_called);
        assert!(!prompter.set_default_called);
    }

    #[test]
    fn configure_existing_provider_user_declines_overwrite_does_not_change_creds() {
        let provider = ProviderCli::WeatherApi;

        let existing_creds = Credentials::WeatherApi {
            api_key: "EXISTING_KEY".to_string(),
        };

        let mut store = InMemoryStore {
            default: Some(provider.into()),
            providers: {
                let mut m = HashMap::new();
                m.insert(provider.into(), existing_creds.clone());
                m
            },
        };

        let mut prompter = MockPrompter {
            overwrite_answer: false,
            set_default_answer: true,
            credentials_to_return: sample_weatherapi_creds(),
            overwrite_called: false,
            set_default_called: false,
            credentials_prompt_called: false,
        };

        ConfigureHandler::new(&mut store, &mut prompter)
            .run(provider)
            .expect("configuration should succeed");

        let saved = store
            .providers
            .get(&provider.into())
            .cloned()
            .expect("credentials must be present");

        assert!(
            saved
                == Credentials::WeatherApi {
                    api_key: "EXISTING_KEY".to_string()
                }
        );
        assert_eq!(store.default, Some(provider.into()));
        assert!(prompter.overwrite_called);
        assert!(!prompter.credentials_prompt_called);
        assert!(!prompter.set_default_called);
    }

    #[test]
    fn configure_existing_provider_user_overwrites_and_changes_default() {
        let provider = ProviderCli::AccuWeather;
        let other = ProviderCli::WeatherApi;

        let existing_creds = Credentials::AccuWeather {
            api_key: "OLD_KEY".to_string(),
        };

        let mut store = InMemoryStore {
            default: Some(other.into()), // some other provider is default
            providers: {
                let mut m = HashMap::new();
                m.insert(provider.into(), existing_creds);
                m
            },
        };

        let mut prompter = MockPrompter {
            overwrite_answer: true,
            set_default_answer: true,
            credentials_to_return: Credentials::AccuWeather {
                api_key: "NEW_KEY".to_string(),
            },
            overwrite_called: false,
            set_default_called: false,
            credentials_prompt_called: false,
        };

        ConfigureHandler::new(&mut store, &mut prompter)
            .run(provider)
            .expect("configuration should succeed");

        let saved = store
            .providers
            .get(&provider.into())
            .cloned()
            .expect("credentials must be present");

        assert!(
            saved
                == Credentials::AccuWeather {
                    api_key: "NEW_KEY".to_string()
                }
        );
        assert_eq!(store.default, Some(provider.into()));
        assert!(prompter.overwrite_called);
        assert!(prompter.credentials_prompt_called);
        assert!(prompter.set_default_called);
    }
}
