use std::fs;
use std::path::{Path, PathBuf};

use directories::ProjectDirs;
use keyring::Entry;
use serde::{Deserialize, Serialize};
use url::Url;
use uuid::Uuid;

use crate::error::{AiError, Result};

const KEYRING_SERVICE: &str = "com.lexito.desktop";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ThemePreference {
    #[default]
    Dark,
    Light,
    System,
}

impl ThemePreference {
    pub const ALL: [ThemePreference; 3] = [Self::Dark, Self::System, Self::Light];

    pub fn label(self) -> &'static str {
        match self {
            ThemePreference::Dark => "Dark",
            ThemePreference::Light => "Light",
            ThemePreference::System => "System",
        }
    }
}

impl std::fmt::Display for ThemePreference {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.label())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ProviderType {
    #[default]
    #[serde(alias = "openai")]
    OpenAI,
    #[serde(alias = "openrouter")]
    OpenRouter,
    #[serde(alias = "anthropic")]
    Anthropic,
}

impl ProviderType {
    pub const ALL: [ProviderType; 3] = [Self::OpenAI, Self::OpenRouter, Self::Anthropic];

    pub fn base_url(&self) -> &'static str {
        match self {
            ProviderType::OpenAI => "https://api.openai.com/v1/",
            ProviderType::OpenRouter => "https://openrouter.ai/api/v1/",
            ProviderType::Anthropic => "https://api.anthropic.com/v1/",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            ProviderType::OpenAI => "OpenAI",
            ProviderType::OpenRouter => "OpenRouter",
            ProviderType::Anthropic => "Anthropic",
        }
    }
}

impl std::fmt::Display for ProviderType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.label())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderProfile {
    pub id: Uuid,
    pub name: String,
    #[serde(default)]
    pub provider_type: ProviderType,
    pub base_url: Url,
    pub model: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TranslationPreferences {
    pub temperature: Option<f32>,
    pub timeout_secs: Option<u64>,
    pub batch_concurrency: Option<usize>,
    #[serde(default = "default_true")]
    pub auto_compile_mo_on_save: bool,
    pub default_locale: Option<String>,
    pub system_prompt: Option<String>,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AppSettings {
    #[serde(default)]
    pub providers: Vec<ProviderProfile>,
    pub active_provider_id: Option<Uuid>,
    #[serde(default)]
    pub translation: TranslationPreferences,
    #[serde(default)]
    pub theme: ThemePreference,
    pub last_opened_path: Option<PathBuf>,
}

#[derive(Debug, Clone)]
pub struct ProviderDraft {
    pub id: Option<Uuid>,
    pub name: String,
    pub provider_type: ProviderType,
    pub model: String,
    pub api_key: String,
}

impl Default for ProviderDraft {
    fn default() -> Self {
        Self {
            id: None,
            name: "OpenAI".to_string(),
            provider_type: ProviderType::OpenAI,
            model: String::new(),
            api_key: String::new(),
        }
    }
}

impl AppSettings {
    pub fn active_provider(&self) -> Option<&ProviderProfile> {
        let active = self.active_provider_id?;
        self.providers.iter().find(|provider| provider.id == active)
    }

    pub fn upsert_provider(&mut self, provider: ProviderProfile) {
        if let Some(existing) = self
            .providers
            .iter_mut()
            .find(|item| item.id == provider.id)
        {
            *existing = provider;
        } else {
            self.providers.push(provider);
        }
    }

    pub fn normalize_providers(&mut self) {
        for provider in &mut self.providers {
            if let Ok(url) = Url::parse(provider.provider_type.base_url()) {
                provider.base_url = url;
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct SettingsStore {
    root: PathBuf,
}

impl Default for SettingsStore {
    fn default() -> Self {
        Self::new().expect("project directories should resolve")
    }
}

impl SettingsStore {
    pub fn new() -> Result<Self> {
        let dirs = ProjectDirs::from("com", "lexito", "lexito")
            .ok_or_else(|| AiError::Settings("could not resolve project directories".into()))?;
        Ok(Self {
            root: dirs.config_dir().to_path_buf(),
        })
    }

    pub fn with_root(root: impl AsRef<Path>) -> Self {
        Self {
            root: root.as_ref().to_path_buf(),
        }
    }

    pub fn path(&self) -> PathBuf {
        self.root.join("settings.toml")
    }

    pub fn load(&self) -> Result<AppSettings> {
        let path = self.path();
        if !path.exists() {
            return Ok(AppSettings::default());
        }

        let content =
            fs::read_to_string(&path).map_err(|error| AiError::Settings(error.to_string()))?;
        let mut settings: AppSettings =
            toml::from_str(&content).map_err(|error| AiError::Settings(error.to_string()))?;
        settings.normalize_providers();
        Ok(settings)
    }

    pub fn save(&self, settings: &AppSettings) -> Result<()> {
        fs::create_dir_all(&self.root).map_err(|error| AiError::Settings(error.to_string()))?;
        let content = toml::to_string_pretty(settings)
            .map_err(|error| AiError::Settings(error.to_string()))?;
        fs::write(self.path(), content).map_err(|error| AiError::Settings(error.to_string()))
    }
}

#[derive(Debug, Clone, Default)]
pub struct SecretStore {
    service: String,
}

impl SecretStore {
    pub fn new() -> Self {
        Self {
            service: KEYRING_SERVICE.to_string(),
        }
    }

    pub fn save_api_key(&self, provider_id: Uuid, api_key: &str) -> Result<()> {
        let entry = Entry::new(&self.service, &provider_id.to_string())
            .map_err(|error| AiError::SecretStore(error.to_string()))?;
        entry
            .set_password(api_key)
            .map_err(|error| AiError::SecretStore(error.to_string()))
    }

    pub fn load_api_key(&self, provider_id: Uuid) -> Result<Option<String>> {
        let entry = Entry::new(&self.service, &provider_id.to_string())
            .map_err(|error| AiError::SecretStore(error.to_string()))?;

        match entry.get_password() {
            Ok(secret) => Ok(Some(secret)),
            Err(error) if error.to_string().contains("NoEntry") => Ok(None),
            Err(error) => Err(AiError::SecretStore(error.to_string())),
        }
    }
}

impl ProviderDraft {
    pub fn into_profile(self) -> Result<(ProviderProfile, String)> {
        let id = self.id.unwrap_or_else(Uuid::new_v4);
        let base_url = Url::parse(self.provider_type.base_url())
            .map_err(|error| AiError::InvalidUrl(error.to_string()))?;
        let profile = ProviderProfile {
            id,
            name: self.name.trim().to_string(),
            provider_type: self.provider_type,
            base_url,
            model: self.model.trim().to_string(),
        };
        Ok((profile, self.api_key))
    }
}
