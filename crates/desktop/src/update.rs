use iced::keyboard::{self, key::Named};
use iced::Task;
use lexito_core::{
    list_projects, validate_translation, CatalogDocument, EntryKey, EntryStatus, Project,
    TranslationPayload,
};
use rfd::FileDialog;

use lexito_ai::{fetch_models, BatchProgressEvent};

use crate::app::LexitoApp;
use crate::tasks::{request_for_entry, save_catalog_task};
use crate::theme::truncate;
use crate::types::{AppScreen, EntryFilter, Message, SettingsSaved, SingleTranslationFinished};

impl LexitoApp {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            // ── Project actions ───────────────────────────────────
            Message::OpenProjectFolder => {
                if let Some(dir) = &self.current_project_dir {
                    #[cfg(target_os = "macos")]
                    let _ = std::process::Command::new("open").arg(dir).spawn();
                    #[cfg(target_os = "linux")]
                    let _ = std::process::Command::new("xdg-open").arg(dir).spawn();
                }
                Task::none()
            }

            // ── Keyboard ─────────────────────────────────────────
            Message::KeyboardEvent(event) => {
                if self.screen != AppScreen::Workspace {
                    return Task::none();
                }
                if let keyboard::Event::KeyPressed { key, modifiers, .. } = event {
                    let cmd = modifiers.command();
                    match (key, cmd) {
                        (keyboard::Key::Named(Named::ArrowDown), false) => {
                            return self.update(Message::SelectNextEntry);
                        }
                        (keyboard::Key::Named(Named::ArrowUp), false) => {
                            return self.update(Message::SelectPrevEntry);
                        }
                        (keyboard::Key::Character(c), true) if c.as_str() == "s" => {
                            return self.update(Message::SavePressed);
                        }
                        (keyboard::Key::Named(Named::Enter), true) => {
                            return self.update(Message::ApplyLocalEdit);
                        }
                        (keyboard::Key::Character(c), true) if c.as_str() == "t" => {
                            return self.update(Message::TranslateSelectedPressed);
                        }
                        _ => {}
                    }
                }
                Task::none()
            }
            Message::SelectNextEntry => {
                self.select_next_entry();
                self.scroll_to_selected()
            }
            Message::SelectPrevEntry => {
                self.select_prev_entry();
                self.scroll_to_selected()
            }

            // ── Navigation ───────────────────────────────────────
            Message::GoToProjects => {
                self.screen = AppScreen::ProjectList;
                self.catalog = None;
                self.current_language = None;
                self.projects = list_projects().unwrap_or_default();
                Task::none()
            }
            Message::GoToDashboard => {
                self.screen = AppScreen::ProjectDashboard;
                self.catalog = None;
                self.current_language = None;
                self.refresh_language_stats();
                Task::none()
            }
            Message::GoToSettings => {
                self.screen = AppScreen::Settings;
                Task::none()
            }

            // ── Project list ─────────────────────────────────────
            Message::OpenProject(dir) => {
                self.status = "Loading project\u{2026}".to_string();
                Task::perform(
                    async move {
                        let project = Project::load(&dir).map_err(|e| e.to_string())?;
                        let stats = project.load_language_stats(&dir);
                        Ok((project, dir, stats))
                    },
                    Message::ProjectLoaded,
                )
            }
            Message::ProjectLoaded(result) => {
                match result {
                    Ok((project, dir, stats)) => {
                        self.status = format!("Opened project: {}", project.project.name);
                        self.current_project = Some(project);
                        self.current_project_dir = Some(dir);
                        self.language_stats = stats;
                        self.screen = AppScreen::ProjectDashboard;
                    }
                    Err(error) => {
                        self.status = format!("Failed to open project: {error}");
                    }
                }
                Task::none()
            }
            Message::DeleteProject(dir) => {
                if let Err(e) = std::fs::remove_dir_all(&dir) {
                    self.status = format!("Failed to delete project: {e}");
                } else {
                    self.status = "Project deleted.".to_string();
                    self.projects = list_projects().unwrap_or_default();
                    if self.current_project_dir.as_ref() == Some(&dir) {
                        self.current_project = None;
                        self.current_project_dir = None;
                    }
                }
                Task::none()
            }

            // ── Create project ───────────────────────────────────
            Message::NewProjectNameChanged(name) => {
                self.new_project_name = name;
                Task::none()
            }
            Message::PickPotFile => Task::perform(
                async {
                    FileDialog::new()
                        .add_filter("POT template", &["pot"])
                        .pick_file()
                },
                Message::PotFilePicked,
            ),
            Message::PotFilePicked(path) => {
                self.new_project_pot_path = path;
                Task::none()
            }
            Message::CreateProject => {
                let name = self.new_project_name.trim().to_string();
                if name.is_empty() {
                    self.status = "Enter a project name.".to_string();
                    return Task::none();
                }
                let Some(pot_path) = self.new_project_pot_path.clone() else {
                    self.status = "Pick a .pot file first.".to_string();
                    return Task::none();
                };
                self.creating_project = true;
                self.status = "Creating project\u{2026}".to_string();
                Task::perform(
                    async move { Project::create(&name, &pot_path).map_err(|e| e.to_string()) },
                    Message::ProjectCreated,
                )
            }
            Message::ProjectCreated(result) => {
                self.creating_project = false;
                match result {
                    Ok((project, dir)) => {
                        self.status = format!("Created project: {}", project.project.name);
                        self.current_project = Some(project);
                        self.current_project_dir = Some(dir);
                        self.language_stats = Vec::new();
                        self.new_project_name.clear();
                        self.new_project_pot_path = None;
                        self.projects = list_projects().unwrap_or_default();
                        self.screen = AppScreen::ProjectDashboard;
                    }
                    Err(error) => {
                        self.status = format!("Failed to create project: {error}");
                    }
                }
                Task::none()
            }

            // ── Project dashboard ────────────────────────────────
            Message::AddLanguage(locale) => {
                let Some(project) = self.current_project.clone() else {
                    return Task::none();
                };
                let Some(dir) = self.current_project_dir.clone() else {
                    return Task::none();
                };
                let locale_code = locale.code.to_string();
                Task::perform(
                    async move {
                        let mut project = project;
                        project
                            .add_language(&locale_code, &dir)
                            .map_err(|e| e.to_string())?;
                        Ok((project, locale_code))
                    },
                    Message::LanguageAdded,
                )
            }
            Message::LanguageAdded(result) => {
                match result {
                    Ok((project, locale)) => {
                        self.current_project = Some(project);
                        self.refresh_language_stats();
                        let stat_count = self.language_stats.len();
                        self.status = format!(
                            "Added language: {locale} ({stat_count} language stats loaded)"
                        );
                    }
                    Err(error) => {
                        self.status = format!("Failed to add language: {error}");
                    }
                }
                Task::none()
            }
            Message::RemoveLanguage(locale) => {
                if let (Some(project), Some(dir)) =
                    (&mut self.current_project, &self.current_project_dir)
                {
                    match project.remove_language(&locale, dir) {
                        Ok(()) => {
                            self.status = format!("Removed language: {locale}");
                            self.refresh_language_stats();
                        }
                        Err(error) => {
                            self.status = format!("Failed to remove language: {error}");
                        }
                    }
                }
                Task::none()
            }
            Message::OpenLanguage(locale) => {
                let Some(project) = &self.current_project else {
                    self.status = "No project loaded.".to_string();
                    return Task::none();
                };
                let Some(dir) = self.current_project_dir.clone() else {
                    self.status = "No project directory set.".to_string();
                    return Task::none();
                };
                let Some(po_path) = project.po_path(&locale, &dir) else {
                    self.status = format!("No .po file configured for '{locale}'");
                    return Task::none();
                };
                if !po_path.exists() {
                    self.status = format!("PO file does not exist: {}", po_path.display());
                    return Task::none();
                }
                let locale_clone = locale;
                self.status = "Loading translations\u{2026}".to_string();
                Task::perform(
                    async move {
                        let doc = CatalogDocument::open_po(&po_path).map_err(|e| e.to_string())?;
                        Ok((doc, locale_clone))
                    },
                    Message::LanguageOpened,
                )
            }
            Message::LanguageOpened(result) => {
                match result {
                    Ok((document, locale)) => {
                        self.locale_input = locale.clone();
                        self.current_language = Some(locale);
                        self.catalog = Some(document);
                        self.select_first_visible_entry();
                        self.screen = AppScreen::Workspace;
                        self.status = "Translations loaded.".to_string();
                    }
                    Err(error) => {
                        self.status = format!("Failed to open translations: {error}");
                    }
                }
                Task::none()
            }

            // ── Workspace (catalog editing) ──────────────────────
            Message::SavePressed => {
                let Some(document) = self.catalog.clone() else {
                    return Task::none();
                };

                if document.session().po_path.is_some() {
                    return save_catalog_task(document, None, true);
                }

                Task::perform(
                    async {
                        FileDialog::new()
                            .add_filter("PO catalog", &["po"])
                            .set_file_name("messages.po")
                            .save_file()
                    },
                    Message::SavePathPicked,
                )
            }
            Message::SavePathPicked(path) => match (self.catalog.clone(), path) {
                (Some(document), Some(path)) => save_catalog_task(document, Some(path), true),
                _ => Task::none(),
            },
            Message::CatalogSaved(result) => {
                match result {
                    Ok(saved) => {
                        self.catalog = Some(saved.document);
                        self.status = saved.message;
                    }
                    Err(error) => {
                        self.status = format!("Save failed: {error}");
                    }
                }
                Task::none()
            }
            Message::SelectEntry(key) => {
                self.selected_key = Some(key);
                self.sync_editor_from_selection();
                Task::none()
            }
            Message::FilterSelected(filter) => {
                self.filter = filter;
                self.select_first_visible_entry();
                Task::none()
            }
            Message::LocaleChanged(locale) => {
                self.locale_input = locale.code.to_string();
                if let Some(document) = self.catalog.as_mut() {
                    document.set_locale(locale.code);
                }
                Task::none()
            }
            Message::SingularChanged(value) => {
                self.singular_input = value;
                Task::none()
            }
            Message::PluralChanged(index, value) => {
                if let Some(slot) = self.plural_inputs.get_mut(index) {
                    *slot = value;
                }
                Task::none()
            }
            Message::ApplyLocalEdit => {
                self.apply_local_edit();
                Task::none()
            }
            Message::ToggleSelectedFuzzy => {
                let is_fuzzy = self
                    .selected_entry()
                    .map(|entry| entry.status == EntryStatus::Fuzzy)
                    .unwrap_or(false);

                if let (Some(document), Some(key)) =
                    (self.catalog.as_mut(), self.selected_key.clone())
                {
                    match document.set_fuzzy(&key, !is_fuzzy) {
                        Ok(()) => {
                            self.sync_editor_from_selection();
                            self.status = if is_fuzzy {
                                "Entry marked reviewed.".to_string()
                            } else {
                                "Entry marked fuzzy.".to_string()
                            };
                        }
                        Err(error) => {
                            self.status = format!("Could not update fuzzy flag: {error}");
                        }
                    }
                }
                Task::none()
            }
            Message::TranslateSelectedPressed => {
                let Some(entry) = self.selected_entry().cloned() else {
                    self.status = "Select an entry first.".to_string();
                    return Task::none();
                };

                match self.build_client() {
                    Ok(client) => {
                        let request = request_for_entry(entry, self.current_locale_value());
                        self.translating = true;
                        self.status = "Requesting AI translation...".to_string();
                        Task::perform(
                            async move {
                                let key = request.key.clone();
                                client
                                    .translate(request)
                                    .await
                                    .map(|response| SingleTranslationFinished { key, response })
                                    .map_err(|error| error.to_string())
                            },
                            Message::SingleTranslationFinished,
                        )
                    }
                    Err(error) => {
                        self.status = error;
                        Task::none()
                    }
                }
            }
            Message::SingleTranslationFinished(result) => {
                self.translating = false;
                match result {
                    Ok(outcome) => self.apply_ai_translation(outcome.key, outcome.response),
                    Err(error) => self.status = format!("AI translation failed: {error}"),
                }
                Task::none()
            }
            Message::BatchTranslateUntranslated => self.start_batch(EntryFilter::Untranslated),
            Message::BatchTranslateFuzzy => self.start_batch(EntryFilter::Fuzzy),
            Message::ApproveAllFuzzy => {
                let Some(document) = self.catalog.as_mut() else {
                    return Task::none();
                };
                let fuzzy_keys: Vec<EntryKey> = document
                    .session()
                    .entries
                    .iter()
                    .filter(|e| e.status == EntryStatus::Fuzzy)
                    .map(|e| e.key.clone())
                    .collect();
                let count = fuzzy_keys.len();
                for key in fuzzy_keys {
                    let _ = document.set_fuzzy(&key, false);
                }
                self.sync_editor_from_selection();
                self.status = format!("Approved {count} fuzzy entries.");
                Task::none()
            }
            Message::BatchProgress(progress) => {
                match progress {
                    BatchProgressEvent::Started { total } => {
                        self.batch_total = total;
                        self.batch_completed = 0;
                        self.status = format!("Translating {total} entries\u{2026}");
                    }
                    BatchProgressEvent::Item {
                        completed,
                        total,
                        item,
                    } => {
                        self.batch_completed = completed;
                        self.batch_total = total;
                        match item.result {
                            Ok(response) => self.apply_ai_translation(item.key, response),
                            Err(error) => {
                                self.status = format!(
                                    "Translating: {completed}/{total}. Last error: {error}"
                                );
                            }
                        }
                    }
                    BatchProgressEvent::Finished { completed, total } => {
                        self.batch_handle = None;
                        self.batch_total = 0;
                        self.batch_completed = 0;
                        self.status = format!("Batch translation finished: {completed}/{total}.");
                    }
                }
                Task::none()
            }
            Message::CancelBatch => {
                if let Some(handle) = self.batch_handle.take() {
                    handle.abort();
                    self.batch_total = 0;
                    self.batch_completed = 0;
                    self.spinner_tick = 0;
                    self.status = "Batch translation canceled.".to_string();
                }
                Task::none()
            }
            Message::SpinnerTick => {
                self.spinner_tick = self.spinner_tick.wrapping_add(1);
                Task::none()
            }

            // ── Provider management ──────────────────────────────
            Message::SelectProvider(id) => {
                self.settings.active_provider_id = Some(id);
                self.editing_provider = None;
                self.status = "Active provider switched.".to_string();
                Task::none()
            }
            Message::AddProvider => {
                self.editing_provider = Some(lexito_ai::ProviderDraft::default());
                self.available_models = Vec::new();
                self.models_error = None;
                Task::none()
            }
            Message::EditProvider(id) => {
                if let Some(profile) = self.settings.providers.iter().find(|p| p.id == id).cloned()
                {
                    let draft = self.load_draft_for_provider(&profile);
                    let has_key = !draft.api_key.is_empty();
                    let provider_type = draft.provider_type;
                    let api_key = draft.api_key.clone();
                    self.editing_provider = Some(draft);
                    self.available_models = Vec::new();
                    self.models_error = None;
                    if has_key {
                        self.models_loading = true;
                        return Task::perform(
                            async move {
                                fetch_models(provider_type, &api_key)
                                    .await
                                    .map_err(|e| e.to_string())
                            },
                            Message::ModelsFetched,
                        );
                    }
                }
                Task::none()
            }
            Message::RemoveProvider(id) => {
                self.settings.providers.retain(|p| p.id != id);
                if self.settings.active_provider_id == Some(id) {
                    self.settings.active_provider_id =
                        self.settings.providers.first().map(|p| p.id);
                }
                if self.editing_provider.as_ref().and_then(|d| d.id) == Some(id) {
                    self.editing_provider = None;
                }
                self.status = "Provider removed.".to_string();
                Task::none()
            }
            Message::CancelEditProvider => {
                self.editing_provider = None;
                self.available_models = Vec::new();
                self.models_error = None;
                Task::none()
            }
            Message::DraftNameChanged(value) => {
                if let Some(draft) = &mut self.editing_provider {
                    draft.name = value;
                }
                Task::none()
            }
            Message::DraftProviderTypeChanged(provider_type) => {
                if let Some(draft) = &mut self.editing_provider {
                    draft.provider_type = provider_type;
                    draft.model = String::new();
                    self.available_models = Vec::new();
                    self.models_error = None;
                    if !draft.api_key.is_empty() {
                        self.models_loading = true;
                        let api_key = draft.api_key.clone();
                        return Task::perform(
                            async move {
                                fetch_models(provider_type, &api_key)
                                    .await
                                    .map_err(|e| e.to_string())
                            },
                            Message::ModelsFetched,
                        );
                    }
                }
                Task::none()
            }
            Message::DraftApiKeyChanged(value) => {
                if let Some(draft) = &mut self.editing_provider {
                    draft.api_key = value;
                }
                Task::none()
            }
            Message::DraftModelSelected(model_info) => {
                if let Some(draft) = &mut self.editing_provider {
                    draft.model = model_info.id;
                }
                Task::none()
            }
            Message::FetchModels => {
                if let Some(draft) = &self.editing_provider {
                    if draft.api_key.trim().is_empty() {
                        self.status = "Enter an API key first.".to_string();
                        return Task::none();
                    }
                    self.models_loading = true;
                    self.models_error = None;
                    let provider_type = draft.provider_type;
                    let api_key = draft.api_key.clone();
                    Task::perform(
                        async move {
                            fetch_models(provider_type, &api_key)
                                .await
                                .map_err(|e| e.to_string())
                        },
                        Message::ModelsFetched,
                    )
                } else {
                    Task::none()
                }
            }
            Message::ModelsFetched(result) => {
                self.models_loading = false;
                match result {
                    Ok(models) => {
                        self.status = format!("Fetched {} models.", models.len());
                        self.available_models = models;
                        self.models_error = None;
                    }
                    Err(error) => {
                        self.available_models = Vec::new();
                        self.models_error = Some(error.clone());
                        self.status = format!("Failed to fetch models: {error}");
                    }
                }
                Task::none()
            }
            Message::SaveProvider => {
                let Some(draft) = self.editing_provider.clone() else {
                    return Task::none();
                };
                let store = self.settings_store.clone();
                let secrets = self.secret_store.clone();
                let mut settings = self.settings.clone();

                Task::perform(
                    async move {
                        let (profile, api_key) = draft.into_profile().map_err(|e| e.to_string())?;
                        secrets
                            .save_api_key(profile.id, &api_key)
                            .map_err(|e| e.to_string())?;
                        settings.upsert_provider(profile.clone());
                        if settings.active_provider_id.is_none() {
                            settings.active_provider_id = Some(profile.id);
                        }
                        store.save(&settings).map_err(|e| e.to_string())?;
                        Ok(SettingsSaved { settings })
                    },
                    Message::ProviderSaved,
                )
            }
            Message::ProviderSaved(result) => {
                match result {
                    Ok(saved) => {
                        self.settings = saved.settings;
                        self.editing_provider = None;
                        self.available_models = Vec::new();
                        self.status = "Provider saved.".to_string();
                    }
                    Err(error) => {
                        self.status = format!("Provider save failed: {error}");
                    }
                }
                Task::none()
            }

            // ── Translation preferences ──────────────────────────
            Message::ThemePreferenceChanged(pref) => {
                self.theme_preference = pref;
                self.resolved_theme = crate::app::resolve_theme(pref);
                Task::none()
            }
            Message::TemperatureChanged(value) => {
                self.temperature_input = value;
                Task::none()
            }
            Message::TimeoutChanged(value) => {
                self.timeout_input = value;
                Task::none()
            }
            Message::ConcurrencyChanged(value) => {
                self.concurrency_input = value;
                Task::none()
            }
            Message::DefaultLocaleChanged(locale) => {
                self.default_locale_input = locale.code.to_string();
                Task::none()
            }
            Message::SystemPromptChanged(value) => {
                self.system_prompt_input = value;
                Task::none()
            }
            Message::AutoCompileToggled(value) => {
                self.auto_compile_on_save = value;
                Task::none()
            }
            Message::SaveSettingsPressed => {
                let translation = match self.translation_preferences() {
                    Ok(preferences) => preferences,
                    Err(error) => {
                        self.status = error;
                        return Task::none();
                    }
                };

                let store = self.settings_store.clone();
                let mut settings = self.settings.clone();
                settings.translation = translation;
                settings.theme = self.theme_preference;

                Task::perform(
                    async move {
                        store.save(&settings).map_err(|e| e.to_string())?;
                        Ok(SettingsSaved { settings })
                    },
                    Message::SettingsSaved,
                )
            }
            Message::SettingsSaved(result) => {
                match result {
                    Ok(saved) => {
                        self.settings = saved.settings;
                        self.status = "Settings saved.".to_string();
                    }
                    Err(error) => {
                        self.status = format!("Settings save failed: {error}");
                    }
                }
                Task::none()
            }
        }
    }

    fn apply_local_edit(&mut self) {
        let Some(key) = self.selected_key.clone() else {
            self.status = "Select an entry first.".to_string();
            return;
        };
        let Some(entry) = self.catalog.as_ref().and_then(|document| {
            document
                .session()
                .entries
                .iter()
                .find(|entry| entry.key == key)
                .cloned()
        }) else {
            return;
        };

        let payload = TranslationPayload {
            singular: self.singular_input.clone(),
            plurals: self.plural_inputs.clone(),
        };

        let warnings = validate_translation(&entry, &payload);
        if !warnings.is_empty() {
            self.status = format!(
                "Edit not applied: {}",
                warnings
                    .iter()
                    .map(|warning| warning.message.clone())
                    .collect::<Vec<_>>()
                    .join("; ")
            );
            return;
        }

        let update_result = if let Some(document) = self.catalog.as_mut() {
            match document.update_translation(&key, payload, false) {
                Ok(_) => document
                    .set_fuzzy(&key, false)
                    .map_err(|error| error.to_string()),
                Err(error) => Err(error.to_string()),
            }
        } else {
            return;
        };

        match update_result {
            Ok(()) => {
                self.sync_editor_from_selection();
                self.status = "Entry updated.".to_string();
            }
            Err(error) => {
                self.status = format!("Could not update entry: {error}");
            }
        }
    }

    fn apply_ai_translation(&mut self, key: EntryKey, response: lexito_ai::TranslationResponse) {
        let Some(existing) = self.catalog.as_ref().and_then(|document| {
            document
                .session()
                .entries
                .iter()
                .find(|entry| entry.key == key)
                .cloned()
        }) else {
            return;
        };

        let payload = TranslationPayload {
            singular: response.singular,
            plurals: if existing.msgid_plural.is_some() {
                response.plurals
            } else {
                Vec::new()
            },
        };

        let warnings = validate_translation(&existing, &payload);
        if !warnings.is_empty() {
            self.status = format!(
                "AI output for '{}' needs review: {}",
                truncate(&existing.msgid, 36),
                warnings
                    .iter()
                    .map(|warning| warning.message.clone())
                    .collect::<Vec<_>>()
                    .join("; ")
            );
            return;
        }

        let update_result = if let Some(document) = self.catalog.as_mut() {
            document
                .update_translation(&key, payload, true)
                .map(|_| {
                    format!(
                        "Applied AI translation via {} / {}.",
                        response.metadata.provider_name, response.metadata.model
                    )
                })
                .map_err(|error| error.to_string())
        } else {
            return;
        };

        match update_result {
            Ok(status) => {
                self.status = status;
                self.sync_editor_from_selection();
            }
            Err(error) => {
                self.status = format!("Could not apply AI translation: {error}");
            }
        }
    }

    fn start_batch(&mut self, filter: EntryFilter) -> Task<Message> {
        let Some(document) = &self.catalog else {
            self.status = "Open a catalog first.".to_string();
            return Task::none();
        };

        let locale = self.current_locale_value();
        if locale.is_empty() {
            self.status = "Set a target locale before using AI translation.".to_string();
            return Task::none();
        }

        let client = match self.build_client() {
            Ok(client) => client,
            Err(error) => {
                self.status = error;
                return Task::none();
            }
        };

        let items = document
            .session()
            .entries
            .iter()
            .filter(|entry| filter.matches(entry))
            .cloned()
            .map(|entry| lexito_ai::BatchItem {
                request: request_for_entry(entry, locale.clone()),
            })
            .collect::<Vec<_>>();

        if items.is_empty() {
            self.status = "No entries match the selected batch filter.".to_string();
            return Task::none();
        }

        let (task, handle) =
            Task::run(client.batch_stream(items), Message::BatchProgress).abortable();
        self.batch_handle = Some(handle);
        task
    }
}
