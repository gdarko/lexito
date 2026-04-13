use iced::{Color, Theme};

fn is_dark(theme: &Theme) -> bool {
    theme.extended_palette().is_dark
}

// ── Surfaces ────────────────────────────────────────────────────
// Dark: Catppuccin Mocha with slight transparency for frosted glass.
// Light: fully opaque, clean whites/grays (no glass — it washes out).

pub fn surface_0(theme: &Theme) -> Color {
    if is_dark(theme) {
        Color {
            r: 0.118,
            g: 0.118,
            b: 0.180,
            a: 0.75,
        }
    } else {
        Color::WHITE
    }
}

pub fn surface_1(theme: &Theme) -> Color {
    if is_dark(theme) {
        Color {
            r: 0.192,
            g: 0.196,
            b: 0.267,
            a: 0.7,
        }
    } else {
        // #f0f2f5
        Color {
            r: 0.941,
            g: 0.949,
            b: 0.961,
            a: 1.0,
        }
    }
}

pub fn surface_2(theme: &Theme) -> Color {
    if is_dark(theme) {
        Color {
            r: 0.271,
            g: 0.278,
            b: 0.353,
            a: 0.75,
        }
    } else {
        // #e4e6eb
        Color {
            r: 0.894,
            g: 0.902,
            b: 0.922,
            a: 1.0,
        }
    }
}

pub fn overlay(theme: &Theme) -> Color {
    if is_dark(theme) {
        Color {
            r: 0.094,
            g: 0.094,
            b: 0.145,
            a: 0.85,
        }
    } else {
        // #ebedf0
        Color {
            r: 0.922,
            g: 0.929,
            b: 0.941,
            a: 1.0,
        }
    }
}

pub fn border(theme: &Theme) -> Color {
    if is_dark(theme) {
        Color {
            r: 0.345,
            g: 0.357,
            b: 0.439,
            a: 1.0,
        }
    } else {
        // #d1d5db
        Color {
            r: 0.820,
            g: 0.835,
            b: 0.859,
            a: 1.0,
        }
    }
}

pub fn glass_border(theme: &Theme) -> Color {
    if is_dark(theme) {
        Color {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: 0.08,
        }
    } else {
        Color {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 0.12,
        }
    }
}

// ── Text ────────────────────────────────────────────────────────

pub fn text_main(theme: &Theme) -> Color {
    if is_dark(theme) {
        Color {
            r: 0.804,
            g: 0.839,
            b: 0.957,
            a: 1.0,
        }
    } else {
        // #1a1a2e — near-black
        Color {
            r: 0.102,
            g: 0.102,
            b: 0.180,
            a: 1.0,
        }
    }
}

pub fn text_muted(theme: &Theme) -> Color {
    if is_dark(theme) {
        Color {
            r: 0.651,
            g: 0.678,
            b: 0.784,
            a: 1.0,
        }
    } else {
        // #5c5f77
        Color {
            r: 0.361,
            g: 0.373,
            b: 0.467,
            a: 1.0,
        }
    }
}

pub fn text_faint(theme: &Theme) -> Color {
    if is_dark(theme) {
        Color {
            r: 0.498,
            g: 0.518,
            b: 0.612,
            a: 1.0,
        }
    } else {
        // #8c8fa1
        Color {
            r: 0.549,
            g: 0.561,
            b: 0.631,
            a: 1.0,
        }
    }
}

// ── Status colors ───────────────────────────────────────────────
// Tuned for good contrast on both dark and light backgrounds.

pub const GREEN: Color = Color {
    r: 0.251,
    g: 0.627,
    b: 0.169,
    a: 1.0,
};
pub const YELLOW: Color = Color {
    r: 0.875,
    g: 0.557,
    b: 0.114,
    a: 1.0,
};
pub const RED: Color = Color {
    r: 0.824,
    g: 0.059,
    b: 0.224,
    a: 1.0,
};
pub const GRAY: Color = Color {
    r: 0.498,
    g: 0.518,
    b: 0.612,
    a: 1.0,
};

// #50C878 emerald green — works on both backgrounds
pub const ACCENT: Color = Color {
    r: 0.314,
    g: 0.784,
    b: 0.471,
    a: 1.0,
};

#[allow(dead_code)]
pub const AMBER: Color = Color {
    r: 0.961,
    g: 0.620,
    b: 0.043,
    a: 1.0,
}; // #F59E0B
