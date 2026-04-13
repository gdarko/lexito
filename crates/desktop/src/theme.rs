use iced::widget::{button, container, scrollable, text_input};
use iced::{Background, Border, Color, Shadow, Theme};
use lexito_core::EntryStatus;

use crate::colors;

// ── Container styles ──────────────────────────────────────────────

pub fn panel_style(theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(colors::surface_1(theme))),
        border: Border {
            radius: 10.0.into(),
            width: 1.0,
            color: colors::glass_border(theme),
        },
        ..container::Style::default()
    }
}

pub fn header_style(theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(colors::surface_1(theme))),
        border: Border {
            radius: 0.0.into(),
            width: 0.0,
            color: Color::TRANSPARENT,
        },
        ..container::Style::default()
    }
}

pub fn status_bar_style(theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(colors::overlay(theme))),
        text_color: Some(colors::text_muted(theme)),
        border: Border {
            radius: 0.0.into(),
            width: 1.0,
            color: colors::glass_border(theme),
        },
        ..container::Style::default()
    }
}

pub fn section_style(theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(colors::surface_1(theme))),
        border: Border {
            radius: 8.0.into(),
            width: 1.0,
            color: colors::glass_border(theme),
        },
        ..container::Style::default()
    }
}

// ── Button styles ────────────────────────────────────────────────

pub fn toolbar_button_style(theme: &Theme, status: button::Status) -> button::Style {
    let base = button::Style {
        background: None,
        text_color: colors::text_muted(theme),
        border: Border {
            radius: 6.0.into(),
            ..Border::default()
        },
        shadow: Shadow::default(),
        ..button::Style::default()
    };
    match status {
        button::Status::Hovered => button::Style {
            background: Some(Background::Color(colors::surface_2(theme))),
            text_color: colors::text_main(theme),
            ..base
        },
        button::Status::Pressed => button::Style {
            background: Some(Background::Color(colors::surface_2(theme))),
            text_color: colors::ACCENT,
            ..base
        },
        button::Status::Disabled => button::Style {
            text_color: colors::text_faint(theme),
            ..base
        },
        _ => base,
    }
}

pub fn entry_button_style(
    is_selected: bool,
) -> impl Fn(&Theme, button::Status) -> button::Style + 'static {
    move |theme, status| {
        let base = button::Style {
            background: if is_selected {
                Some(Background::Color(colors::surface_2(theme)))
            } else {
                None
            },
            text_color: if is_selected {
                colors::text_main(theme)
            } else {
                colors::text_muted(theme)
            },
            border: Border {
                radius: 4.0.into(),
                ..Border::default()
            },
            shadow: Shadow::default(),
            ..button::Style::default()
        };
        match status {
            button::Status::Hovered if !is_selected => button::Style {
                background: Some(Background::Color(Color {
                    a: 0.3,
                    ..colors::surface_1(theme)
                })),
                text_color: colors::text_main(theme),
                ..base
            },
            _ => base,
        }
    }
}

pub fn filter_button_style(
    is_active: bool,
) -> impl Fn(&Theme, button::Status) -> button::Style + 'static {
    move |theme, status| {
        let base = button::Style {
            background: Some(Background::Color(if is_active {
                colors::ACCENT
            } else {
                colors::surface_2(theme)
            })),
            text_color: if is_active {
                colors::surface_0(theme)
            } else {
                colors::text_muted(theme)
            },
            border: Border {
                radius: 12.0.into(),
                width: if is_active { 0.0 } else { 1.0 },
                color: colors::glass_border(theme),
            },
            shadow: Shadow::default(),
            ..button::Style::default()
        };
        match status {
            button::Status::Hovered if !is_active => button::Style {
                text_color: colors::text_main(theme),
                ..base
            },
            _ => base,
        }
    }
}

pub fn accent_button_style(theme: &Theme, status: button::Status) -> button::Style {
    let base = button::Style {
        background: Some(Background::Color(colors::ACCENT)),
        text_color: Color::WHITE,
        border: Border {
            radius: 6.0.into(),
            ..Border::default()
        },
        shadow: Shadow::default(),
        ..button::Style::default()
    };
    match status {
        button::Status::Hovered => button::Style {
            background: Some(Background::Color(Color {
                r: 0.259,
                g: 0.698,
                b: 0.400,
                a: 1.0,
            })),
            ..base
        },
        button::Status::Pressed => button::Style {
            background: Some(Background::Color(Color {
                r: 0.220,
                g: 0.620,
                b: 0.350,
                a: 1.0,
            })),
            ..base
        },
        button::Status::Disabled => button::Style {
            background: Some(Background::Color(colors::surface_2(theme))),
            text_color: colors::text_faint(theme),
            ..base
        },
        _ => base,
    }
}

pub fn secondary_button_style(theme: &Theme, status: button::Status) -> button::Style {
    let base = button::Style {
        background: Some(Background::Color(colors::surface_2(theme))),
        text_color: colors::text_main(theme),
        border: Border {
            radius: 6.0.into(),
            width: 1.0,
            color: colors::glass_border(theme),
        },
        shadow: Shadow::default(),
        ..button::Style::default()
    };
    match status {
        button::Status::Hovered => button::Style {
            text_color: colors::ACCENT,
            ..base
        },
        button::Status::Pressed => button::Style {
            background: Some(Background::Color(colors::surface_1(theme))),
            text_color: colors::ACCENT,
            ..base
        },
        button::Status::Disabled => button::Style {
            text_color: colors::text_faint(theme),
            ..base
        },
        _ => base,
    }
}

// ── Input style ──────────────────────────────────────────────────

pub fn input_style(theme: &Theme, status: text_input::Status) -> text_input::Style {
    let active = text_input::Style {
        background: Background::Color(colors::surface_0(theme)),
        border: Border {
            radius: 6.0.into(),
            width: 1.0,
            color: colors::glass_border(theme),
        },
        icon: colors::text_muted(theme),
        placeholder: colors::text_faint(theme),
        value: colors::text_main(theme),
        selection: Color {
            a: 0.3,
            ..colors::ACCENT
        },
    };
    match status {
        text_input::Status::Active => active,
        text_input::Status::Hovered => {
            let hover_border = colors::border(theme);
            text_input::Style {
                border: Border {
                    color: Color {
                        a: 0.5,
                        ..hover_border
                    },
                    ..active.border
                },
                ..active
            }
        }
        text_input::Status::Focused { .. } => text_input::Style {
            border: Border {
                color: Color {
                    a: 0.6,
                    ..colors::ACCENT
                },
                ..active.border
            },
            ..active
        },
        text_input::Status::Disabled => text_input::Style {
            background: Background::Color(colors::surface_1(theme)),
            value: colors::text_faint(theme),
            ..active
        },
    }
}

// ── Provider card style ──────────────────────────────────────────

pub fn provider_card_style(
    is_active: bool,
) -> impl Fn(&Theme, button::Status) -> button::Style + 'static {
    move |theme, status| {
        let border_color = if is_active {
            Color {
                a: 0.5,
                ..colors::ACCENT
            }
        } else {
            colors::glass_border(theme)
        };
        let base = button::Style {
            background: Some(Background::Color(if is_active {
                Color {
                    a: 0.15,
                    ..colors::ACCENT
                }
            } else {
                colors::surface_1(theme)
            })),
            text_color: colors::text_main(theme),
            border: Border {
                radius: 8.0.into(),
                width: if is_active { 1.5 } else { 1.0 },
                color: border_color,
            },
            shadow: Shadow::default(),
            ..button::Style::default()
        };
        match status {
            button::Status::Hovered if !is_active => button::Style {
                background: Some(Background::Color(colors::surface_2(theme))),
                border: Border {
                    color: Color {
                        a: 0.3,
                        ..colors::ACCENT
                    },
                    ..base.border
                },
                ..base
            },
            _ => base,
        }
    }
}

// ── Scrollbar style ─────────────────────────────────────────────

pub fn accent_scrollbar_style(theme: &Theme, _status: scrollable::Status) -> scrollable::Style {
    let scroller_color = Color {
        a: 0.35,
        ..colors::ACCENT
    };
    let rail = scrollable::Rail {
        background: None,
        border: Border::default(),
        scroller: scrollable::Scroller {
            background: Background::Color(scroller_color),
            border: Border {
                radius: 4.0.into(),
                ..Border::default()
            },
        },
    };
    scrollable::Style {
        container: container::Style::default(),
        vertical_rail: rail,
        horizontal_rail: rail,
        gap: None,
        auto_scroll: scrollable::AutoScroll {
            background: Background::Color(colors::surface_2(theme)),
            border: Border::default(),
            shadow: Shadow::default(),
            icon: colors::text_muted(theme),
        },
    }
}

// ── Progress bar style ───────────────────────────────────────────

pub fn accent_progress_style(theme: &Theme) -> iced::widget::progress_bar::Style {
    iced::widget::progress_bar::Style {
        background: Background::Color(colors::surface_2(theme)),
        bar: Background::Color(colors::ACCENT),
        border: Border {
            radius: 3.0.into(),
            ..Border::default()
        },
    }
}

// ── Combo box menu style ─────────────────────────────────────────

pub fn combo_menu_style(theme: &Theme) -> iced::widget::overlay::menu::Style {
    let bg = colors::surface_1(theme);
    let bg_opaque = Color { a: 1.0, ..bg };
    iced::widget::overlay::menu::Style {
        background: Background::Color(bg_opaque),
        border: Border {
            color: colors::border(theme),
            width: 1.0,
            radius: 8.0.into(),
        },
        text_color: colors::text_main(theme),
        selected_text_color: Color::WHITE,
        selected_background: Background::Color(colors::ACCENT),
        shadow: Shadow::default(),
    }
}

// ── Helpers ──────────────────────────────────────────────────────

pub fn status_color(status: EntryStatus) -> Color {
    match status {
        EntryStatus::Translated => colors::GREEN,
        EntryStatus::Untranslated => colors::RED,
        EntryStatus::Fuzzy => colors::YELLOW,
        EntryStatus::Obsolete => colors::GRAY,
    }
}

pub fn status_pill(status: EntryStatus) -> container::Style {
    let color = status_color(status);
    container::Style {
        background: Some(Background::Color(Color { a: 0.15, ..color })),
        border: Border {
            radius: 4.0.into(),
            ..Border::default()
        },
        text_color: Some(color),
        ..container::Style::default()
    }
}

pub fn status_label(status: EntryStatus) -> &'static str {
    match status {
        EntryStatus::Translated => "Translated",
        EntryStatus::Untranslated => "Untranslated",
        EntryStatus::Fuzzy => "Fuzzy",
        EntryStatus::Obsolete => "Obsolete",
    }
}

pub fn truncate(value: &str, max: usize) -> String {
    if value.chars().count() <= max {
        return value.to_string();
    }

    value.chars().take(max).collect::<String>() + "..."
}
