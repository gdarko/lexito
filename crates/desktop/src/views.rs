use iced::widget::{
    button, checkbox, column, combo_box, container, opaque, pick_list, progress_bar, row, rule,
    scrollable, space, stack, text, text_input,
};
use iced::{Background, Border, Color, Element, Font, Length, Padding, Theme};
use lexito_ai::{ProviderProfile, ProviderType, ThemePreference};
use lexito_core::EntryStatus;

use crate::app::LexitoApp;
use crate::colors;
use crate::icons;
use crate::locales::Locale;
use crate::theme::{
    accent_button_style, accent_progress_style, accent_scrollbar_style, entry_button_style,
    filter_button_style, header_style, input_style, panel_style, provider_card_style,
    secondary_button_style, section_style, status_bar_style, status_color, status_label,
    status_pill, toolbar_button_style, truncate,
};
use crate::types::{AppScreen, EntryFilter, Message};

impl LexitoApp {
    pub fn view(&self) -> Element<'_, Message> {
        match self.screen {
            AppScreen::ProjectList => self.view_project_list(),
            AppScreen::ProjectDashboard => self.view_project_dashboard(),
            AppScreen::Workspace => self.view_workspace_screen(),
            AppScreen::Settings => self.view_settings_screen(),
        }
    }

    fn view_workspace_screen(&self) -> Element<'_, Message> {
        let th = &self.resolved_theme;
        let header = self.view_workspace_header();
        let content = self.view_workspace();

        let bottom =
            container(text(&self.status).size(12).color(colors::text_muted(th)))
                .style(status_bar_style)
                .padding([6, 16])
                .width(Length::Fill);

        let base = column![header, content, bottom]
            .width(Length::Fill)
            .height(Length::Fill);

        if self.batch_handle.is_some() && self.batch_total > 0 {
            const SPINNER: &[char] = &['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];
            let frame = SPINNER[self.spinner_tick % SPINNER.len()];
            let pct = self.batch_completed as f32 / self.batch_total as f32;
            let label = format!(
                "{frame} Translating: {}/{} ({:.0}%)",
                self.batch_completed,
                self.batch_total,
                pct * 100.0
            );

            let modal_content = container(
                column![
                    text(label).size(14).color(colors::text_main(th)),
                    progress_bar(0.0..=1.0, pct)
                        .girth(6)
                        .style(accent_progress_style),
                    button(text("Cancel").size(13))
                        .style(secondary_button_style)
                        .padding([6, 20])
                        .on_press(Message::CancelBatch),
                ]
                .spacing(12)
                .align_x(iced::Alignment::Center)
                .width(360),
            )
            .padding(24)
            .style(section_style)
            .center_x(Length::Fill)
            .center_y(Length::Fill);

            let backdrop_color = colors::overlay(th);
            let overlay = container(opaque(modal_content))
                .style(move |_: &Theme| container::Style {
                    background: Some(Background::Color(backdrop_color)),
                    ..Default::default()
                })
                .width(Length::Fill)
                .height(Length::Fill);

            stack![base, overlay].into()
        } else {
            base.into()
        }
    }

    fn view_settings_screen(&self) -> Element<'_, Message> {
        let th = &self.resolved_theme;

        let back_target = if self.current_project.is_some() {
            Message::GoToDashboard
        } else {
            Message::GoToProjects
        };

        let header = container(
            row![
                button(text("\u{2190} Back").size(13))
                    .style(toolbar_button_style)
                    .padding([6, 12])
                    .on_press(back_target),
                text("Settings").size(16).color(colors::text_main(th)),
            ]
            .spacing(12)
            .align_y(iced::Alignment::Center),
        )
        .style(header_style)
        .padding(header_padding())
        .center_y(48.0 + crate::TITLEBAR_TOP_PAD)
        .width(Length::Fill);

        let content = self.view_settings();

        let status_bar = container(text(&self.status).size(12).color(colors::text_muted(th)))
            .style(status_bar_style)
            .padding([6, 16])
            .width(Length::Fill);

        column![header, content, status_bar]
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn view_workspace_header(&self) -> Element<'_, Message> {
        let th = &self.resolved_theme;

        let project_name = self
            .current_project
            .as_ref()
            .map(|p| p.project.name.as_str())
            .unwrap_or("lexito");
        let lang_label = self.current_language.as_deref().unwrap_or("");

        let nav = row![
            button(
                row![icons::arrow_left(14), text("Dashboard").size(13)]
                    .spacing(6)
                    .align_y(iced::Alignment::Center)
            )
            .style(toolbar_button_style)
            .padding([6, 12])
            .on_press(Message::GoToDashboard),
            text(project_name).size(14).color(colors::text_main(th)),
            text(lang_label).size(14).color(colors::ACCENT),
        ]
        .spacing(8)
        .align_y(iced::Alignment::Center);

        let mut bulk = row![].spacing(4);
        if self.catalog.is_some() {
            bulk = bulk.push(
                button(
                    row![icons::translate(14), text("Translate All").size(12)]
                        .spacing(6)
                        .align_y(iced::Alignment::Center),
                )
                .style(toolbar_button_style)
                .padding([6, 12])
                .on_press(Message::BatchTranslateUntranslated),
            );
            bulk = bulk.push(
                button(
                    row![icons::refresh(14), text("Retranslate Fuzzy").size(12)]
                        .spacing(6)
                        .align_y(iced::Alignment::Center),
                )
                .style(toolbar_button_style)
                .padding([6, 12])
                .on_press(Message::BatchTranslateFuzzy),
            );
            bulk = bulk.push(
                button(
                    row![icons::check(14), text("Approve All").size(12)]
                        .spacing(6)
                        .align_y(iced::Alignment::Center),
                )
                .style(toolbar_button_style)
                .padding([6, 12])
                .on_press(Message::ApproveAllFuzzy),
            );
        }

        let mut file_actions = row![].spacing(4);
        if self.catalog.is_some() {
            file_actions = file_actions.push(
                button(
                    row![icons::save(14), text("Save").size(13)]
                        .spacing(6)
                        .align_y(iced::Alignment::Center),
                )
                .style(toolbar_button_style)
                .padding([6, 12])
                .on_press(Message::SavePressed),
            );
        }

        container(
            row![nav, space::horizontal(), bulk, file_actions]
                .spacing(8)
                .align_y(iced::Alignment::Center),
        )
        .style(header_style)
        .padding(header_padding())
        .center_y(48.0 + crate::TITLEBAR_TOP_PAD)
        .width(Length::Fill)
        .into()
    }

    fn view_workspace(&self) -> Element<'_, Message> {
        let th = &self.resolved_theme;

        let Some(document) = &self.catalog else {
            return container(
                column![
                    text("No catalog loaded")
                        .size(24)
                        .color(colors::text_muted(th)),
                    text("Select a language from the project dashboard to start translating.")
                        .size(14)
                        .color(colors::text_faint(th)),
                    button(text("Back to Dashboard").size(14))
                        .style(accent_button_style)
                        .padding([10, 24])
                        .on_press(Message::GoToDashboard),
                ]
                .spacing(16)
                .align_x(iced::Alignment::Center),
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .into();
        };

        let session = document.session();

        // Stats badges
        let stats = row![
            stat_badge(session.stats.translated, "translated", colors::GREEN, th),
            stat_badge(session.stats.untranslated, "untranslated", colors::RED, th),
            stat_badge(session.stats.fuzzy, "fuzzy", colors::YELLOW, th),
            stat_badge(session.stats.warnings, "warnings", colors::RED, th),
        ]
        .spacing(6);

        // Filter pills
        let mut filters = row![].spacing(4);
        for filter in [
            EntryFilter::All,
            EntryFilter::Untranslated,
            EntryFilter::Fuzzy,
            EntryFilter::Warnings,
            EntryFilter::Obsolete,
        ] {
            let is_active = self.filter == filter;
            filters = filters.push(
                button(text(filter.label()).size(12))
                    .style(filter_button_style(is_active))
                    .padding([4, 10])
                    .on_press(Message::FilterSelected(filter)),
            );
        }

        // Entry list with colored status dots
        let mut entry_list = column![].spacing(2);
        for entry in session
            .entries
            .iter()
            .filter(|entry| self.filter.matches(entry))
        {
            let selected = self
                .selected_key
                .as_ref()
                .map(|key| key == &entry.key)
                .unwrap_or(false);

            let color = status_color(entry.status);
            let dot = text("\u{25CF}").size(10).color(color);
            let label = text(truncate(&entry.msgid, 36))
                .size(13)
                .color(if selected {
                    colors::text_main(th)
                } else {
                    colors::text_muted(th)
                });

            let mut entry_row = row![dot, label].spacing(8).align_y(iced::Alignment::Center);

            if !entry.warnings.is_empty() {
                entry_row = entry_row.push(text("!").size(12).color(colors::RED));
            }

            entry_list = entry_list.push(
                button(entry_row)
                    .style(entry_button_style(selected))
                    .padding([6, 10])
                    .width(Length::Fill)
                    .on_press(Message::SelectEntry(entry.key.clone())),
            );
        }

        let sidebar = container(
            column![
                text("Catalog").size(16).color(colors::ACCENT),
                combo_box(
                    &self.locale_state,
                    "Search locale\u{2026}",
                    find_locale(&self.locale_input),
                    Message::LocaleChanged,
                ),
                text(format!("Source: {}", session.source_path.display()))
                    .size(12)
                    .color(colors::text_faint(th)),
                stats,
                divider(th),
                filters,
                scrollable(entry_list)
                    .id(self.entry_scroll_id.clone())
                    .style(accent_scrollbar_style)
                    .height(Length::Fill),
            ]
            .spacing(12),
        )
        .style(panel_style)
        .width(Length::FillPortion(2))
        .height(Length::Fill)
        .padding(16);

        let editor = self.view_entry_editor();

        row![sidebar, editor]
            .spacing(8)
            .height(Length::Fill)
            .padding([8, 8])
            .into()
    }

    fn view_entry_editor(&self) -> Element<'_, Message> {
        let th = &self.resolved_theme;

        let Some(entry) = self.selected_entry() else {
            return container(
                column![
                    text("Select an entry")
                        .size(20)
                        .color(colors::text_muted(th)),
                    text("Choose a translation entry from the sidebar to begin editing")
                        .size(14)
                        .color(colors::text_faint(th)),
                ]
                .spacing(8)
                .align_x(iced::Alignment::Center),
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .into();
        };

        let status = entry.status;

        // ── Source section (read-only reference) ─────────────

        let status_pill_widget = container(
            text(status_label(status))
                .size(12)
                .color(status_color(status)),
        )
        .padding([2, 8])
        .style(move |_theme| status_pill(status));

        let mut source_content = column![
            row![
                text("Source").size(14).color(colors::text_faint(th)),
                space::horizontal(),
                status_pill_widget,
            ]
            .align_y(iced::Alignment::Center),
            text(&entry.msgid)
                .size(15)
                .color(colors::text_main(th))
                .font(Font::with_name("JetBrains Mono")),
        ]
        .spacing(8);

        if let Some(msgid_plural) = &entry.msgid_plural {
            source_content = source_content.push(
                row![
                    text("Plural:").size(12).color(colors::text_faint(th)),
                    text(msgid_plural).size(13).color(colors::text_muted(th)),
                ]
                .spacing(6),
            );
        }

        if let Some(msgctxt) = &entry.msgctxt {
            source_content = source_content.push(
                row![
                    text("Context:").size(12).color(colors::text_faint(th)),
                    text(msgctxt).size(13).color(colors::text_muted(th)),
                ]
                .spacing(6),
            );
        }

        // Compact metadata row
        let mut meta_parts: Vec<Element<'_, Message>> = Vec::new();
        if !entry.references.is_empty() {
            meta_parts.push(
                row![
                    text("Refs:").size(11).color(colors::text_faint(th)),
                    text(entry.references.join(", "))
                        .size(11)
                        .color(colors::text_faint(th)),
                ]
                .spacing(4)
                .into(),
            );
        }
        if !entry.extracted_comment.trim().is_empty() {
            meta_parts.push(
                row![
                    text("Comment:").size(11).color(colors::text_faint(th)),
                    text(&entry.extracted_comment)
                        .size(11)
                        .color(colors::text_faint(th)),
                ]
                .spacing(4)
                .into(),
            );
        }
        if !entry.flags.is_empty() {
            meta_parts.push(
                row![
                    text("Flags:").size(11).color(colors::text_faint(th)),
                    text(entry.flags.join(", "))
                        .size(11)
                        .color(colors::text_faint(th)),
                ]
                .spacing(4)
                .into(),
            );
        }

        if !meta_parts.is_empty() {
            let mut meta_col = column![].spacing(2);
            for part in meta_parts {
                meta_col = meta_col.push(part);
            }
            source_content = source_content.push(meta_col);
        }

        let source_section = container(source_content)
            .style(section_style)
            .padding(16)
            .width(Length::Fill);

        // ── Translation section (main action area) ──────────

        let action_buttons = row![
            button(
                row![icons::sparkles(14), text("AI Translate").size(13)]
                    .spacing(6)
                    .align_y(iced::Alignment::Center)
            )
            .style(accent_button_style)
            .padding([8, 20])
            .on_press(Message::TranslateSelectedPressed),
            button(
                row![icons::check(14), text("Apply").size(13)]
                    .spacing(6)
                    .align_y(iced::Alignment::Center)
            )
            .style(secondary_button_style)
            .padding([8, 16])
            .on_press(Message::ApplyLocalEdit),
        ]
        .spacing(8);

        let has_plural = entry.msgid_plural.is_some();

        let mut translation_content = column![row![
            text("Translation").size(14).color(colors::ACCENT),
            space::horizontal(),
            action_buttons,
        ]
        .align_y(iced::Alignment::Center),]
        .spacing(12);

        if has_plural {
            // Singular form with label
            translation_content = translation_content.push(
                column![
                    text("Singular (n = 1)")
                        .size(12)
                        .color(colors::text_faint(th)),
                    text_input("Singular translation\u{2026}", &self.singular_input)
                        .style(input_style)
                        .on_input(Message::SingularChanged)
                        .padding(12),
                ]
                .spacing(4),
            );

            // Plural forms with labels
            for (index, value) in self.plural_inputs.iter().enumerate() {
                let label = match index {
                    0 => "Plural form 0 (e.g. n = 0 or n >= 2)".to_string(),
                    1 => "Plural form 1 (e.g. n = 2\u{2013}4)".to_string(),
                    n => format!("Plural form {n}"),
                };
                translation_content = translation_content.push(
                    column![
                        text(label).size(12).color(colors::text_faint(th)),
                        text_input("Plural translation\u{2026}", value)
                            .style(input_style)
                            .on_input(move |next| Message::PluralChanged(index, next))
                            .padding(12),
                    ]
                    .spacing(4),
                );
            }
        } else {
            // Simple singular entry
            translation_content = translation_content.push(
                text_input("Enter translation\u{2026}", &self.singular_input)
                    .style(input_style)
                    .on_input(Message::SingularChanged)
                    .padding(12),
            );
        }

        // Secondary action
        translation_content = translation_content.push(
            button(
                text(if entry.status == EntryStatus::Fuzzy {
                    "Mark reviewed"
                } else {
                    "Mark fuzzy"
                })
                .size(12),
            )
            .style(toolbar_button_style)
            .padding([4, 12])
            .on_press(Message::ToggleSelectedFuzzy),
        );

        // Warnings
        if !entry.warnings.is_empty() {
            let mut warnings = column![].spacing(4);
            for warning in &entry.warnings {
                warnings = warnings.push(
                    row![
                        text("\u{25CF}").size(8).color(colors::RED),
                        text(&warning.message).size(12).color(colors::RED),
                    ]
                    .spacing(6)
                    .align_y(iced::Alignment::Center),
                );
            }
            translation_content = translation_content.push(
                container(warnings)
                    .style(|_theme| container::Style {
                        background: Some(Background::Color(Color {
                            a: 0.1,
                            ..colors::RED
                        })),
                        border: Border {
                            radius: 6.0.into(),
                            ..Border::default()
                        },
                        ..container::Style::default()
                    })
                    .padding(10)
                    .width(Length::Fill),
            );
        }

        let translation_section = container(translation_content)
            .style(section_style)
            .padding(16)
            .width(Length::Fill);

        // ── Assemble ─────────────────────────────────────────

        container(scrollable(
            column![source_section, translation_section].spacing(12),
        ))
        .width(Length::FillPortion(5))
        .height(Length::Fill)
        .padding(8)
        .into()
    }

    fn view_settings(&self) -> Element<'_, Message> {
        let th = &self.resolved_theme;

        let provider_cards = self.view_provider_cards();

        let mut settings_body = column![provider_cards].spacing(16);

        if let Some(editor) = self.view_provider_editor() {
            settings_body = settings_body.push(editor);
        }

        // ── Appearance section ───────────────────────────────
        let theme_section = container(
            column![
                text("Appearance").size(16).color(colors::ACCENT),
                divider(th),
                row![
                    theme_pill(
                        "Dark",
                        self.theme_preference == ThemePreference::Dark,
                        Message::ThemePreferenceChanged(ThemePreference::Dark),
                    ),
                    theme_pill(
                        "System",
                        self.theme_preference == ThemePreference::System,
                        Message::ThemePreferenceChanged(ThemePreference::System),
                    ),
                    theme_pill(
                        "Light",
                        self.theme_preference == ThemePreference::Light,
                        Message::ThemePreferenceChanged(ThemePreference::Light),
                    ),
                ]
                .spacing(8),
            ]
            .spacing(12),
        )
        .style(section_style)
        .padding(20)
        .width(Length::Fill);

        // ── Translation preferences section ──────────────────
        let translation_section = container(
            column![
                text("Translation Preferences")
                    .size(16)
                    .color(colors::ACCENT),
                divider(th),
                settings_field(
                    "Temperature",
                    &self.temperature_input,
                    Message::TemperatureChanged,
                    th,
                ),
                settings_field(
                    "Timeout (seconds)",
                    &self.timeout_input,
                    Message::TimeoutChanged,
                    th,
                ),
                settings_field(
                    "Batch concurrency",
                    &self.concurrency_input,
                    Message::ConcurrencyChanged,
                    th,
                ),
                column![
                    text("Default locale")
                        .size(12)
                        .color(colors::text_muted(th)),
                    combo_box(
                        &self.default_locale_state,
                        "Search locale\u{2026}",
                        find_locale(&self.default_locale_input),
                        Message::DefaultLocaleChanged,
                    )
                ]
                .spacing(4),
                settings_field(
                    "System prompt",
                    &self.system_prompt_input,
                    Message::SystemPromptChanged,
                    th,
                ),
                checkbox(self.auto_compile_on_save)
                    .label("Auto compile .mo on save")
                    .on_toggle(Message::AutoCompileToggled),
            ]
            .spacing(12),
        )
        .style(section_style)
        .padding(20)
        .width(Length::Fill);

        let save_btn = button(text("Save Preferences").size(14))
            .style(accent_button_style)
            .padding([10, 24])
            .on_press(Message::SaveSettingsPressed);

        settings_body = settings_body.push(theme_section);
        settings_body = settings_body.push(translation_section);
        settings_body = settings_body.push(save_btn);

        container(scrollable(settings_body.max_width(640).padding(24)))
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .into()
    }

    fn view_provider_cards(&self) -> Element<'_, Message> {
        let th = &self.resolved_theme;

        let mut cards = column![
            row![
                text("AI Providers").size(16).color(colors::ACCENT),
                space::horizontal(),
                button(text("+").size(16))
                    .style(accent_button_style)
                    .padding([4, 12])
                    .on_press(Message::AddProvider),
            ]
            .align_y(iced::Alignment::Center),
            divider(th),
        ]
        .spacing(12);

        for provider in &self.settings.providers {
            let is_active = self.settings.active_provider_id == Some(provider.id);
            cards = cards.push(self.view_provider_card(provider, is_active));
        }

        if self.settings.providers.is_empty() {
            cards = cards.push(
                text("No providers configured. Click + to add one.")
                    .size(13)
                    .color(colors::text_faint(th)),
            );
        }

        container(cards)
            .style(section_style)
            .padding(20)
            .width(Length::Fill)
            .into()
    }

    fn view_provider_card<'a>(
        &self,
        provider: &'a ProviderProfile,
        is_active: bool,
    ) -> Element<'a, Message> {
        let th = &self.resolved_theme;
        let id = provider.id;

        let name_label = text(&provider.name).size(14).color(colors::text_main(th));
        let info = row![
            text(provider.provider_type.label())
                .size(12)
                .color(colors::text_faint(th)),
            text(" / ").size(12).color(colors::text_faint(th)),
            text(&provider.model).size(12).color(colors::text_muted(th)),
        ];

        let active_label = if is_active {
            text("Active").size(11).color(colors::GREEN)
        } else {
            text("").size(11)
        };

        let actions = row![
            button(text("Edit").size(11))
                .style(toolbar_button_style)
                .padding([2, 8])
                .on_press(Message::EditProvider(id)),
            button(text("Remove").size(11))
                .style(toolbar_button_style)
                .padding([2, 8])
                .on_press(Message::RemoveProvider(id)),
        ]
        .spacing(4);

        let card_content = row![
            column![name_label, info].spacing(2),
            space::horizontal(),
            column![active_label, actions]
                .spacing(4)
                .align_x(iced::Alignment::End),
        ]
        .align_y(iced::Alignment::Center);

        button(card_content)
            .style(provider_card_style(is_active))
            .padding([12, 16])
            .width(Length::Fill)
            .on_press(Message::SelectProvider(id))
            .into()
    }

    fn view_provider_editor(&self) -> Option<Element<'_, Message>> {
        let th = &self.resolved_theme;
        let draft = self.editing_provider.as_ref()?;

        let type_picker = pick_list(
            &ProviderType::ALL[..],
            Some(draft.provider_type),
            Message::DraftProviderTypeChanged,
        );

        let name_field = text_input("Provider name", &draft.name)
            .style(input_style)
            .on_input(Message::DraftNameChanged)
            .padding(10);

        let api_key_field = text_input("API Key", &draft.api_key)
            .style(input_style)
            .on_input(Message::DraftApiKeyChanged)
            .padding(10)
            .secure(true);

        let fetch_label = if self.models_loading {
            "Loading\u{2026}"
        } else {
            "Fetch models"
        };
        let fetch_btn = button(text(fetch_label).size(12))
            .style(secondary_button_style)
            .padding([4, 10]);
        let fetch_btn = if self.models_loading {
            fetch_btn
        } else {
            fetch_btn.on_press(Message::FetchModels)
        };

        let selected_model = self
            .available_models
            .iter()
            .find(|m| m.id == draft.model)
            .cloned();

        let model_picker = pick_list(
            self.available_models.clone(),
            selected_model,
            Message::DraftModelSelected,
        )
        .placeholder("Select a model\u{2026}");

        let title = if draft.id.is_some() {
            "Edit Provider"
        } else {
            "Add Provider"
        };

        let mut editor = column![
            text(title).size(16).color(colors::ACCENT),
            divider(th),
            column![
                text("Type").size(12).color(colors::text_muted(th)),
                type_picker
            ]
            .spacing(4),
            column![
                text("Name").size(12).color(colors::text_muted(th)),
                name_field
            ]
            .spacing(4),
            column![
                text("API Key").size(12).color(colors::text_muted(th)),
                api_key_field
            ]
            .spacing(4),
            row![
                column![
                    text("Model").size(12).color(colors::text_muted(th)),
                    model_picker
                ]
                .spacing(4)
                .width(Length::Fill),
                fetch_btn
            ]
            .spacing(8)
            .align_y(iced::Alignment::End),
        ]
        .spacing(12);

        if let Some(error) = &self.models_error {
            editor = editor.push(text(error).size(12).color(colors::RED));
        }

        let actions = row![
            button(text("Save").size(13))
                .style(accent_button_style)
                .padding([6, 16])
                .on_press(Message::SaveProvider),
            button(text("Cancel").size(13))
                .style(secondary_button_style)
                .padding([6, 16])
                .on_press(Message::CancelEditProvider),
        ]
        .spacing(8);

        editor = editor.push(actions);

        Some(
            container(editor)
                .style(section_style)
                .padding(20)
                .width(Length::Fill)
                .into(),
        )
    }
}

// ── View helper functions ────────────────────────────────────────

fn header_padding() -> Padding {
    Padding {
        top: crate::TITLEBAR_TOP_PAD,
        bottom: 0.0,
        left: crate::TITLEBAR_LEFT_PAD.max(16.0),
        right: 16.0,
    }
}

fn find_locale(code: &str) -> Option<&'static Locale> {
    crate::locales::ALL_LOCALES.iter().find(|l| l.code == code)
}

fn divider<'a>(theme: &Theme) -> Element<'a, Message> {
    let color = colors::border(theme);
    rule::horizontal(1)
        .style(move |_theme| rule::Style {
            color,
            radius: 0.0.into(),
            fill_mode: rule::FillMode::Full,
            snap: true,
        })
        .into()
}

fn stat_badge<'a>(count: usize, label: &str, color: Color, theme: &Theme) -> Element<'a, Message> {
    let surface = colors::surface_2(theme);
    container(
        row![
            text(count.to_string()).size(14).color(color),
            text(label.to_string())
                .size(11)
                .color(colors::text_muted(theme)),
        ]
        .spacing(4)
        .align_y(iced::Alignment::Center),
    )
    .padding([4, 8])
    .style(move |_theme| container::Style {
        background: Some(Background::Color(surface)),
        border: Border {
            radius: 4.0.into(),
            ..Border::default()
        },
        ..container::Style::default()
    })
    .into()
}

fn settings_field<'a>(
    label: &'a str,
    value: &'a str,
    on_input: fn(String) -> Message,
    theme: &Theme,
) -> Element<'a, Message> {
    column![
        text(label).size(12).color(colors::text_muted(theme)),
        text_input("", value)
            .style(input_style)
            .on_input(on_input)
            .padding(10),
    ]
    .spacing(4)
    .into()
}

fn theme_pill<'a>(label: &'a str, is_active: bool, message: Message) -> Element<'a, Message> {
    button(text(label).size(13))
        .style(filter_button_style(is_active))
        .padding([6, 16])
        .on_press(message)
        .into()
}
