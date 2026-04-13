use iced::widget::{
    button, column, combo_box, container, progress_bar, row, scrollable, space, text,
};
use iced::{Element, Length, Padding};

use crate::app::LexitoApp;
use crate::colors;
use crate::icons;
use crate::locales::Locale;
use crate::theme::{
    accent_button_style, accent_progress_style, combo_menu_style, header_style, input_style,
    secondary_button_style, section_style, status_bar_style, toolbar_button_style,
};
use crate::types::Message;

impl LexitoApp {
    pub fn view_project_list(&self) -> Element<'_, Message> {
        let th = &self.resolved_theme;

        let header = container(
            row![
                icons::languages(20),
                text("lexito").size(16).color(colors::ACCENT),
                space::horizontal(),
                button(icons::settings(16))
                    .style(toolbar_button_style)
                    .padding([6, 12])
                    .on_press(Message::GoToSettings),
            ]
            .align_y(iced::Alignment::Center),
        )
        .style(header_style)
        .padding(header_padding())
        .center_y(48.0 + crate::TITLEBAR_TOP_PAD)
        .width(Length::Fill);

        let content = if self.projects.is_empty() {
            self.view_empty_project_list()
        } else {
            self.view_populated_project_list()
        };

        let status_bar = container(text(&self.status).size(12).color(colors::text_muted(th)))
            .style(status_bar_style)
            .padding([6, 16])
            .width(Length::Fill);

        column![header, content, status_bar]
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn view_empty_project_list(&self) -> Element<'_, Message> {
        let th = &self.resolved_theme;

        let pot_label = match &self.new_project_pot_path {
            Some(path) => path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("selected")
                .to_string(),
            None => String::new(),
        };

        let pot_status = if self.new_project_pot_path.is_some() {
            row![
                text("\u{2713}").size(13).color(colors::GREEN),
                text(pot_label.clone())
                    .size(13)
                    .color(colors::text_muted(th)),
            ]
            .spacing(4)
        } else {
            row![text("No template selected")
                .size(13)
                .color(colors::text_faint(th)),]
        };

        let create_form = container(
            column![
                text("Create your first project")
                    .size(24)
                    .color(colors::text_main(th)),
                text("Start by giving your project a name and selecting a .pot template file.")
                    .size(14)
                    .color(colors::text_faint(th)),
                column![
                    text("Project name").size(12).color(colors::text_muted(th)),
                    iced::widget::text_input("e.g. My WordPress Plugin", &self.new_project_name)
                        .style(input_style)
                        .on_input(Message::NewProjectNameChanged)
                        .padding(12),
                ]
                .spacing(4),
                column![
                    text("Source template")
                        .size(12)
                        .color(colors::text_muted(th)),
                    row![
                        button(text("Choose .pot file").size(13))
                            .style(secondary_button_style)
                            .padding([10, 20])
                            .on_press(Message::PickPotFile),
                        pot_status,
                    ]
                    .spacing(12)
                    .align_y(iced::Alignment::Center),
                ]
                .spacing(4),
                button(text("Create Project").size(14))
                    .style(accent_button_style)
                    .padding([12, 32])
                    .on_press(Message::CreateProject),
            ]
            .spacing(20),
        )
        .style(section_style)
        .padding(32)
        .max_width(480);

        container(create_form)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .into()
    }

    fn view_populated_project_list(&self) -> Element<'_, Message> {
        let th = &self.resolved_theme;

        let pot_label = match &self.new_project_pot_path {
            Some(path) => path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("selected")
                .to_string(),
            None => String::new(),
        };

        let pot_indicator = if self.new_project_pot_path.is_some() {
            text(pot_label.clone())
                .size(12)
                .color(colors::text_muted(th))
        } else {
            text("").size(12)
        };

        // ── New project row ──────────────────────────────
        let create_row = row![
            iced::widget::text_input("New project name", &self.new_project_name)
                .style(input_style)
                .on_input(Message::NewProjectNameChanged)
                .padding(10)
                .width(Length::Fill),
            button(text("Pick .pot").size(13))
                .style(secondary_button_style)
                .padding([8, 16])
                .on_press(Message::PickPotFile),
            pot_indicator,
            button(
                row![icons::plus(14), text("Create").size(13)]
                    .spacing(6)
                    .align_y(iced::Alignment::Center)
            )
            .style(accent_button_style)
            .padding([8, 16])
            .on_press(Message::CreateProject),
        ]
        .spacing(8)
        .align_y(iced::Alignment::Center);

        // ── Project cards ────────────────────────────────
        let mut project_cards = column![].spacing(8);
        for (name, dir) in &self.projects {
            let dir_open = dir.clone();
            let dir_delete = dir.clone();

            let lang_count = std::fs::read_dir(dir)
                .map(|entries| {
                    entries
                        .flatten()
                        .filter(|e| e.path().extension().map(|ext| ext == "po").unwrap_or(false))
                        .count()
                })
                .unwrap_or(0);

            let lang_label = match lang_count {
                0 => "No languages".to_string(),
                1 => "1 language".to_string(),
                n => format!("{n} languages"),
            };

            let card = button(
                row![
                    column![
                        text(name).size(15).color(colors::text_main(th)),
                        text(lang_label).size(12).color(colors::text_faint(th)),
                    ]
                    .spacing(2),
                    space::horizontal(),
                    button(text("Delete").size(11))
                        .style(toolbar_button_style)
                        .padding([4, 8])
                        .on_press(Message::DeleteProject(dir_delete)),
                ]
                .align_y(iced::Alignment::Center),
            )
            .style(crate::theme::provider_card_style(false))
            .padding([14, 20])
            .width(Length::Fill)
            .on_press(Message::OpenProject(dir_open));

            project_cards = project_cards.push(card);
        }

        // ── Layout: centered, wide ───────────────────────
        let body = column![
            row![
                text("Projects").size(22).color(colors::text_main(th)),
                space::horizontal(),
            ]
            .align_y(iced::Alignment::Center),
            create_row,
            project_cards,
        ]
        .spacing(16)
        .max_width(800);

        container(scrollable(container(body).padding(32)).height(Length::Fill))
            .width(Length::Fill)
            .center_x(Length::Fill)
            .into()
    }

    pub fn view_project_dashboard(&self) -> Element<'_, Message> {
        let th = &self.resolved_theme;

        let Some(project) = &self.current_project else {
            return text("No project loaded").into();
        };

        let header = container(
            row![
                button(text("\u{2190} Projects").size(13))
                    .style(toolbar_button_style)
                    .padding([6, 12])
                    .on_press(Message::GoToProjects),
                text(&project.project.name)
                    .size(18)
                    .color(colors::text_main(th)),
                space::horizontal(),
                button(
                    row![icons::save(14), text("Open Folder").size(12)]
                        .spacing(6)
                        .align_y(iced::Alignment::Center),
                )
                .style(toolbar_button_style)
                .padding([6, 12])
                .on_press(Message::OpenProjectFolder),
                button(icons::settings(16))
                    .style(toolbar_button_style)
                    .padding([6, 12])
                    .on_press(Message::GoToSettings),
            ]
            .spacing(8)
            .align_y(iced::Alignment::Center),
        )
        .style(header_style)
        .padding(header_padding())
        .center_y(48.0 + crate::TITLEBAR_TOP_PAD)
        .width(Length::Fill);

        // Project summary
        let total_entries: usize = self
            .language_stats
            .iter()
            .map(|(_, s)| s.total)
            .max()
            .unwrap_or(0);
        let lang_count = project.languages.len();
        let summary = row![
            text(format!(
                "{lang_count} language{}",
                if lang_count == 1 { "" } else { "s" }
            ))
            .size(13)
            .color(colors::text_muted(th)),
            text("\u{2022}").size(13).color(colors::text_faint(th)),
            text(format!("{total_entries} entries"))
                .size(13)
                .color(colors::text_muted(th)),
        ]
        .spacing(8);

        // Add language
        let add_language_section = container(
            column![
                text("Languages").size(16).color(colors::ACCENT),
                summary,
                row![
                    icons::plus(16),
                    combo_box(
                        &self.add_language_state,
                        "Add a language\u{2026}",
                        None::<&Locale>,
                        Message::AddLanguage,
                    )
                    .menu_style(combo_menu_style)
                    .width(Length::Fill),
                ]
                .spacing(8)
                .align_y(iced::Alignment::Center),
            ]
            .spacing(12),
        )
        .style(section_style)
        .padding(20)
        .width(Length::Fill);

        // Language cards
        let mut lang_cards = column![].spacing(12);

        if project.languages.is_empty() {
            lang_cards = lang_cards.push(
                container(
                    column![
                        icons::languages(32),
                        text("No languages yet")
                            .size(18)
                            .color(colors::text_muted(th)),
                        text("Search and add your first target language above.")
                            .size(14)
                            .color(colors::text_faint(th)),
                    ]
                    .spacing(12)
                    .align_x(iced::Alignment::Center),
                )
                .width(Length::Fill)
                .padding([48, 0])
                .center_x(Length::Fill),
            );
        }

        for lang in &project.languages {
            let stats = self
                .language_stats
                .iter()
                .find(|(l, _)| l == &lang.locale)
                .map(|(_, s)| s);

            let (pct, translated, total, fuzzy) = if let Some(s) = stats {
                let pct = if s.total > 0 {
                    (s.translated as f64 / s.total as f64 * 100.0) as u32
                } else {
                    0
                };
                (pct, s.translated, s.total, s.fuzzy)
            } else {
                (0, 0, 0, 0)
            };

            let pct_color = if pct == 100 {
                colors::GREEN
            } else if pct >= 50 {
                colors::YELLOW
            } else {
                colors::RED
            };

            let locale_code = lang.locale.clone();
            let locale_remove = lang.locale.clone();

            let display_name = crate::locales::ALL_LOCALES
                .iter()
                .find(|l| l.code == lang.locale)
                .map(|l| l.name)
                .unwrap_or("");

            let pct_f32 = pct as f32 / 100.0;

            let card = button(
                column![
                    row![
                        column![
                            row![
                                text(&lang.locale).size(20).color(colors::ACCENT),
                                text(display_name).size(15).color(colors::text_main(th)),
                            ]
                            .spacing(8)
                            .align_y(iced::Alignment::Center),
                            row![
                                text(format!("{translated}/{total} translated"))
                                    .size(12)
                                    .color(colors::text_faint(th)),
                                if fuzzy > 0 {
                                    text(format!("{fuzzy} fuzzy"))
                                        .size(12)
                                        .color(colors::YELLOW)
                                } else {
                                    text("").size(12)
                                },
                            ]
                            .spacing(8),
                        ]
                        .spacing(4),
                        space::horizontal(),
                        text(format!("{pct}%")).size(24).color(pct_color),
                        button(text("Remove").size(11))
                            .style(toolbar_button_style)
                            .padding([4, 8])
                            .on_press(Message::RemoveLanguage(locale_remove)),
                    ]
                    .align_y(iced::Alignment::Center),
                    progress_bar(0.0..=1.0, pct_f32)
                        .girth(6)
                        .style(accent_progress_style),
                ]
                .spacing(12),
            )
            .style(crate::theme::provider_card_style(false))
            .padding([20, 24])
            .width(Length::Fill)
            .on_press(Message::OpenLanguage(locale_code));

            lang_cards = lang_cards.push(card);
        }

        let status_bar = container(text(&self.status).size(12).color(colors::text_muted(th)))
            .style(status_bar_style)
            .padding([6, 16])
            .width(Length::Fill);

        column![
            header,
            container(
                scrollable(
                    container(
                        column![add_language_section, lang_cards]
                            .spacing(16)
                            .max_width(800)
                    )
                    .padding(32)
                )
                .height(Length::Fill)
            )
            .width(Length::Fill)
            .center_x(Length::Fill),
            status_bar,
        ]
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }
}

fn header_padding() -> Padding {
    Padding {
        top: crate::TITLEBAR_TOP_PAD,
        bottom: 0.0,
        left: crate::TITLEBAR_LEFT_PAD.max(16.0),
        right: 16.0,
    }
}
