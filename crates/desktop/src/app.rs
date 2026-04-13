use std::path::PathBuf;

use iced::task;
use iced::widget::{combo_box, scrollable};
use iced::{keyboard, time, widget, Subscription, Task, Theme};
use lexito_ai::{
    AiClient, AppSettings, ModelInfo, ProviderDraft, ProviderProfile, ResolvedProviderProfile,
    SecretStore, SettingsStore, ThemePreference, TranslationPreferences,
};
use lexito_core::{CatalogDocument, CatalogEntry, CatalogStats, EntryKey, Project};

use crate::locales::{all_locales, Locale};
use crate::types::{AppScreen, EntryFilter, Message};

pub struct LexitoApp {
    pub(crate) settings_store: SettingsStore,
    pub(crate) secret_store: SecretStore,
    pub(crate) settings: AppSettings,
    // Theme
    pub(crate) theme_preference: ThemePreference,
    pub(crate) resolved_theme: Theme,
    // Screen navigation
    pub(crate) screen: AppScreen,
    pub(crate) status: String,
    // Project list
    pub(crate) projects: Vec<(String, PathBuf)>,
    // Current project
    pub(crate) current_project: Option<Project>,
    pub(crate) current_project_dir: Option<PathBuf>,
    pub(crate) language_stats: Vec<(String, CatalogStats)>,
    pub(crate) current_language: Option<String>,
    // Create project dialog
    pub(crate) new_project_name: String,
    pub(crate) new_project_pot_path: Option<PathBuf>,
    pub(crate) creating_project: bool,
    // Add language
    pub(crate) add_language_state: combo_box::State<Locale>,
    // Workspace (catalog editing)
    pub(crate) entry_scroll_id: widget::Id,
    pub(crate) catalog: Option<CatalogDocument>,
    pub(crate) selected_key: Option<EntryKey>,
    pub(crate) filter: EntryFilter,
    pub(crate) locale_input: String,
    pub(crate) locale_state: combo_box::State<Locale>,
    pub(crate) singular_input: String,
    pub(crate) plural_inputs: Vec<String>,
    pub(crate) batch_handle: Option<task::Handle>,
    pub(crate) batch_total: usize,
    pub(crate) batch_completed: usize,
    pub(crate) spinner_tick: usize,
    // Provider editing
    pub(crate) editing_provider: Option<ProviderDraft>,
    pub(crate) available_models: Vec<ModelInfo>,
    pub(crate) models_loading: bool,
    pub(crate) models_error: Option<String>,
    // Translation preferences
    pub(crate) temperature_input: String,
    pub(crate) timeout_input: String,
    pub(crate) concurrency_input: String,
    pub(crate) default_locale_input: String,
    pub(crate) default_locale_state: combo_box::State<Locale>,
    pub(crate) system_prompt_input: String,
    pub(crate) auto_compile_on_save: bool,
}

pub(crate) fn resolve_theme(preference: ThemePreference) -> Theme {
    match preference {
        ThemePreference::Dark => Theme::CatppuccinMocha,
        ThemePreference::Light => Theme::CatppuccinLatte,
        ThemePreference::System => match dark_light::detect() {
            Ok(dark_light::Mode::Light) => Theme::CatppuccinLatte,
            _ => Theme::CatppuccinMocha,
        },
    }
}

impl LexitoApp {
    pub fn boot() -> (Self, Task<Message>) {
        let settings_store =
            SettingsStore::new().unwrap_or_else(|_| SettingsStore::with_root(".lexito"));
        let secret_store = SecretStore::new();
        let settings = settings_store.load().unwrap_or_default();

        let projects = lexito_core::list_projects().unwrap_or_default();

        let temperature_input = settings
            .translation
            .temperature
            .map(|value| value.to_string())
            .unwrap_or_else(|| "0.2".to_string());
        let timeout_input = settings
            .translation
            .timeout_secs
            .map(|value| value.to_string())
            .unwrap_or_else(|| "60".to_string());
        let concurrency_input = settings
            .translation
            .batch_concurrency
            .map(|value| value.to_string())
            .unwrap_or_else(|| "4".to_string());
        let default_locale_input = settings
            .translation
            .default_locale
            .clone()
            .unwrap_or_default();
        let system_prompt_input = settings
            .translation
            .system_prompt
            .clone()
            .unwrap_or_default();
        let auto_compile_on_save = settings.translation.auto_compile_mo_on_save;
        let theme_preference = settings.theme;
        let resolved_theme = resolve_theme(theme_preference);

        (
            Self {
                settings_store,
                secret_store,
                settings,
                theme_preference,
                resolved_theme,
                screen: AppScreen::ProjectList,
                status: String::new(),
                projects,
                current_project: None,
                current_project_dir: None,
                language_stats: Vec::new(),
                current_language: None,
                new_project_name: String::new(),
                new_project_pot_path: None,
                creating_project: false,
                add_language_state: combo_box::State::new(all_locales()),
                entry_scroll_id: widget::Id::unique(),
                catalog: None,
                selected_key: None,
                filter: EntryFilter::All,
                locale_input: default_locale_input.clone(),
                locale_state: combo_box::State::new(all_locales()),
                singular_input: String::new(),
                plural_inputs: Vec::new(),
                batch_handle: None,
                batch_total: 0,
                batch_completed: 0,
                spinner_tick: 0,
                editing_provider: None,
                available_models: Vec::new(),
                models_loading: false,
                models_error: None,
                temperature_input,
                timeout_input,
                concurrency_input,
                default_locale_input,
                default_locale_state: combo_box::State::new(all_locales()),
                system_prompt_input,
                auto_compile_on_save,
            },
            Task::none(),
        )
    }

    pub fn title(&self) -> String {
        match &self.current_project {
            Some(project) => {
                if let Some(lang) = &self.current_language {
                    format!("lexito - {} ({})", project.project.name, lang)
                } else {
                    format!("lexito - {}", project.project.name)
                }
            }
            None => "lexito".to_string(),
        }
    }

    pub fn theme(&self) -> Theme {
        self.resolved_theme.clone()
    }

    pub(crate) fn current_locale_value(&self) -> String {
        let locale = self.locale_input.trim();
        if !locale.is_empty() {
            locale.to_string()
        } else {
            self.default_locale_input.trim().to_string()
        }
    }

    pub(crate) fn selected_entry(&self) -> Option<&CatalogEntry> {
        let document = self.catalog.as_ref()?;
        let key = self.selected_key.as_ref()?;
        document
            .session()
            .entries
            .iter()
            .find(|entry| &entry.key == key)
    }

    pub(crate) fn sync_editor_from_selection(&mut self) {
        if let Some(entry) = self.selected_entry().cloned() {
            self.singular_input = entry.msgstr.clone();
            self.plural_inputs = if entry.msgid_plural.is_some() {
                if entry.msgstr_plural.is_empty() {
                    vec![String::new(), String::new()]
                } else {
                    entry.msgstr_plural.clone()
                }
            } else {
                Vec::new()
            };
        }
    }

    pub(crate) fn select_first_visible_entry(&mut self) {
        let next = self.catalog.as_ref().and_then(|document| {
            document
                .session()
                .entries
                .iter()
                .find(|entry| self.filter.matches(entry))
                .map(|entry| entry.key.clone())
        });
        self.selected_key = next;
        self.sync_editor_from_selection();
    }

    pub(crate) fn build_client(&self) -> Result<AiClient, String> {
        let provider = self
            .settings
            .active_provider()
            .ok_or_else(|| "No active provider configured. Add one in Settings.".to_string())?;
        let api_key = self
            .secret_store
            .load_api_key(provider.id)
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "No API key stored for the active provider.".to_string())?;
        let resolved = ResolvedProviderProfile::from((provider.clone(), api_key));
        let preferences = self.translation_preferences()?;
        AiClient::new(resolved, preferences).map_err(|e| e.to_string())
    }

    pub(crate) fn load_draft_for_provider(&self, profile: &ProviderProfile) -> ProviderDraft {
        let api_key = self
            .secret_store
            .load_api_key(profile.id)
            .ok()
            .flatten()
            .unwrap_or_default();
        ProviderDraft {
            id: Some(profile.id),
            name: profile.name.clone(),
            provider_type: profile.provider_type,
            model: profile.model.clone(),
            api_key,
        }
    }

    pub(crate) fn translation_preferences(&self) -> Result<TranslationPreferences, String> {
        let temperature = self
            .temperature_input
            .trim()
            .parse::<f32>()
            .map_err(|_| "Temperature must be a number.".to_string())?;
        let timeout_secs = self
            .timeout_input
            .trim()
            .parse::<u64>()
            .map_err(|_| "Timeout must be an integer number of seconds.".to_string())?;
        let batch_concurrency = self
            .concurrency_input
            .trim()
            .parse::<usize>()
            .map_err(|_| "Concurrency must be a whole number.".to_string())?;

        Ok(TranslationPreferences {
            temperature: Some(temperature),
            timeout_secs: Some(timeout_secs),
            batch_concurrency: Some(batch_concurrency.max(1)),
            auto_compile_mo_on_save: self.auto_compile_on_save,
            default_locale: (!self.default_locale_input.trim().is_empty())
                .then_some(self.default_locale_input.trim().to_string()),
            system_prompt: (!self.system_prompt_input.trim().is_empty())
                .then_some(self.system_prompt_input.trim().to_string()),
        })
    }

    pub(crate) fn refresh_language_stats(&mut self) {
        if let (Some(project), Some(dir)) = (&self.current_project, &self.current_project_dir) {
            self.language_stats = project.load_language_stats(dir);
        }
    }

    pub fn subscription(&self) -> Subscription<Message> {
        let keys = keyboard::listen().map(Message::KeyboardEvent);

        if self.batch_handle.is_some() {
            let spinner = time::every(std::time::Duration::from_millis(80))
                .map(|_| Message::SpinnerTick);
            Subscription::batch([keys, spinner])
        } else {
            keys
        }
    }

    pub(crate) fn select_next_entry(&mut self) {
        let Some(document) = &self.catalog else {
            return;
        };
        let filtered: Vec<&CatalogEntry> = document
            .session()
            .entries
            .iter()
            .filter(|e| self.filter.matches(e))
            .collect();
        if filtered.is_empty() {
            return;
        }
        let current_idx = self
            .selected_key
            .as_ref()
            .and_then(|key| filtered.iter().position(|e| &e.key == key))
            .unwrap_or(0);
        let next_idx = (current_idx + 1).min(filtered.len() - 1);
        self.selected_key = Some(filtered[next_idx].key.clone());
        self.sync_editor_from_selection();
    }

    pub(crate) fn select_prev_entry(&mut self) {
        let Some(document) = &self.catalog else {
            return;
        };
        let filtered: Vec<&CatalogEntry> = document
            .session()
            .entries
            .iter()
            .filter(|e| self.filter.matches(e))
            .collect();
        if filtered.is_empty() {
            return;
        }
        let current_idx = self
            .selected_key
            .as_ref()
            .and_then(|key| filtered.iter().position(|e| &e.key == key))
            .unwrap_or(0);
        let prev_idx = current_idx.saturating_sub(1);
        self.selected_key = Some(filtered[prev_idx].key.clone());
        self.sync_editor_from_selection();
    }

    pub(crate) fn scroll_to_selected(&self) -> Task<Message> {
        let Some(document) = &self.catalog else {
            return Task::none();
        };
        let filtered: Vec<_> = document
            .session()
            .entries
            .iter()
            .filter(|e| self.filter.matches(e))
            .collect();
        if filtered.is_empty() {
            return Task::none();
        }
        let idx = self
            .selected_key
            .as_ref()
            .and_then(|key| filtered.iter().position(|e| &e.key == key))
            .unwrap_or(0);
        let pct = if filtered.len() <= 1 {
            0.0
        } else {
            idx as f32 / (filtered.len() - 1) as f32
        };
        iced::widget::operation::snap_to(
            self.entry_scroll_id.clone(),
            scrollable::RelativeOffset { x: 0.0, y: pct },
        )
    }
}
