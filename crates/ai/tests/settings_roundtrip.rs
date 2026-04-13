use std::str::FromStr;

use lexito_ai::{
    AppSettings, ProviderProfile, ProviderType, SettingsStore, ThemePreference,
    TranslationPreferences,
};
use tempfile::tempdir;
use url::Url;
use uuid::Uuid;

#[test]
fn persists_settings_roundtrip() {
    let tempdir = tempdir().expect("tempdir");
    let store = SettingsStore::with_root(tempdir.path());

    let provider = ProviderProfile {
        id: Uuid::new_v4(),
        name: "Primary".to_string(),
        provider_type: ProviderType::OpenAI,
        base_url: Url::from_str("https://api.openai.com/v1/").expect("url"),
        model: "gpt-4.1-mini".to_string(),
    };

    let settings = AppSettings {
        providers: vec![provider.clone()],
        active_provider_id: Some(provider.id),
        translation: TranslationPreferences {
            temperature: Some(0.2),
            timeout_secs: Some(45),
            batch_concurrency: Some(3),
            auto_compile_mo_on_save: true,
            default_locale: Some("fr".to_string()),
            system_prompt: Some("Keep tone formal.".to_string()),
        },
        theme: ThemePreference::Light,
        last_opened_path: None,
    };

    store.save(&settings).expect("save");
    let loaded = store.load().expect("load");

    assert_eq!(loaded.active_provider_id, settings.active_provider_id);
    assert_eq!(loaded.providers[0].base_url, settings.providers[0].base_url);
    assert_eq!(loaded.providers[0].provider_type, ProviderType::OpenAI);
    assert_eq!(loaded.translation.default_locale, Some("fr".to_string()));
    assert_eq!(loaded.theme, ThemePreference::Light);
}

#[test]
fn loads_legacy_settings_without_provider_type() {
    let tempdir = tempdir().expect("tempdir");
    let store = SettingsStore::with_root(tempdir.path());

    let toml_content = r#"
[[providers]]
id = "550e8400-e29b-41d4-a716-446655440000"
name = "Primary"
base_url = "https://api.openai.com/v1/"
model = "gpt-4.1-mini"
"#;
    std::fs::create_dir_all(tempdir.path()).unwrap();
    std::fs::write(tempdir.path().join("settings.toml"), toml_content).unwrap();

    let settings = store.load().expect("load");
    assert_eq!(settings.providers[0].provider_type, ProviderType::OpenAI);
    assert_eq!(settings.providers[0].name, "Primary");
}

#[test]
fn roundtrips_multiple_provider_types() {
    let tempdir = tempdir().expect("tempdir");
    let store = SettingsStore::with_root(tempdir.path());

    let settings = AppSettings {
        providers: vec![
            ProviderProfile {
                id: Uuid::new_v4(),
                name: "My OpenAI".to_string(),
                provider_type: ProviderType::OpenAI,
                base_url: Url::parse("https://api.openai.com/v1/").unwrap(),
                model: "gpt-4.1-mini".to_string(),
            },
            ProviderProfile {
                id: Uuid::new_v4(),
                name: "My Anthropic".to_string(),
                provider_type: ProviderType::Anthropic,
                base_url: Url::parse("https://api.anthropic.com/v1/").unwrap(),
                model: "claude-sonnet-4-20250514".to_string(),
            },
            ProviderProfile {
                id: Uuid::new_v4(),
                name: "My OpenRouter".to_string(),
                provider_type: ProviderType::OpenRouter,
                base_url: Url::parse("https://openrouter.ai/api/v1/").unwrap(),
                model: "anthropic/claude-sonnet-4".to_string(),
            },
        ],
        active_provider_id: None,
        translation: TranslationPreferences::default(),
        theme: ThemePreference::default(),
        last_opened_path: None,
    };

    store.save(&settings).expect("save");
    let loaded = store.load().expect("load");

    assert_eq!(loaded.providers.len(), 3);
    assert_eq!(loaded.providers[0].provider_type, ProviderType::OpenAI);
    assert_eq!(loaded.providers[1].provider_type, ProviderType::Anthropic);
    assert_eq!(loaded.providers[2].provider_type, ProviderType::OpenRouter);
}
