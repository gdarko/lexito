use std::collections::BTreeSet;
use std::sync::LazyLock;

use regex::Regex;

use crate::model::{CatalogEntry, TranslationPayload, ValidationCode, ValidationWarning};

static PLACEHOLDER_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"%(\(\w+\))?[#0\- +'I]*(\d+\$)?[0-9.*hlLjzt]*[a-zA-Z%]|\{[a-zA-Z0-9_.-]+\}")
        .expect("placeholder regex")
});

static TAG_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"</?[A-Za-z][^>]*>").expect("tag regex"));

fn extract_tokens(pattern: &Regex, values: impl Iterator<Item = String>) -> BTreeSet<String> {
    values
        .flat_map(|value| {
            pattern
                .find_iter(&value)
                .map(|capture| capture.as_str().to_string())
                .collect::<Vec<_>>()
        })
        .collect()
}

pub fn validate_translation(
    entry: &CatalogEntry,
    translation: &TranslationPayload,
) -> Vec<ValidationWarning> {
    let mut warnings = Vec::new();

    let source_tokens = extract_tokens(
        &PLACEHOLDER_RE,
        std::iter::once(entry.msgid.clone())
            .chain(entry.msgid_plural.clone())
            .collect::<Vec<_>>()
            .into_iter(),
    );
    let target_tokens = extract_tokens(
        &PLACEHOLDER_RE,
        std::iter::once(translation.singular.clone())
            .chain(translation.plurals.clone())
            .collect::<Vec<_>>()
            .into_iter(),
    );

    if source_tokens != target_tokens {
        warnings.push(ValidationWarning {
            code: ValidationCode::PlaceholderMismatch,
            message: "The translation changed or dropped placeholders.".to_string(),
        });
    }

    let source_tags = extract_tokens(
        &TAG_RE,
        std::iter::once(entry.msgid.clone())
            .chain(entry.msgid_plural.clone())
            .collect::<Vec<_>>()
            .into_iter(),
    );
    let target_tags = extract_tokens(
        &TAG_RE,
        std::iter::once(translation.singular.clone())
            .chain(translation.plurals.clone())
            .collect::<Vec<_>>()
            .into_iter(),
    );

    if source_tags != target_tags {
        warnings.push(ValidationWarning {
            code: ValidationCode::TagMismatch,
            message: "The translation changed or dropped HTML/XML-like tags.".to_string(),
        });
    }

    let expected_plural_count = usize::from(entry.msgid_plural.is_some());
    if entry.msgid_plural.is_some() && translation.plurals.is_empty() {
        warnings.push(ValidationWarning {
            code: ValidationCode::PluralMismatch,
            message: "Plural source entry requires plural translations.".to_string(),
        });
    }

    if expected_plural_count == 0 && !translation.plurals.is_empty() {
        warnings.push(ValidationWarning {
            code: ValidationCode::PluralMismatch,
            message: "Singular source entry should not produce plural translations.".to_string(),
        });
    }

    warnings
}
