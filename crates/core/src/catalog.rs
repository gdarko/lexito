use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use rspolib::prelude::*;
use rspolib::{pofile, POEntry, POFile};

use crate::error::{CatalogError, Result};
use crate::model::{
    CatalogEntry, CatalogHeader, CatalogSession, CatalogSourceKind, CatalogStats, EntryKey,
    EntryStatus, SaveOutcome, TranslationPayload,
};
use crate::validation::validate_translation;

#[derive(Debug, Clone)]
pub struct CatalogDocument {
    session: CatalogSession,
    po_file: POFile,
}

impl CatalogDocument {
    pub fn open(path: impl AsRef<Path>, locale: Option<String>) -> Result<Self> {
        let path = path.as_ref();
        match path.extension().and_then(|ext| ext.to_str()) {
            Some("po") => Self::open_po(path),
            Some("pot") => Self::open_pot(path, locale),
            _ => Err(CatalogError::UnsupportedSource(path.to_path_buf())),
        }
    }

    pub fn open_po(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let po_file = read_po(path)?;
        Ok(Self::from_po_file(
            CatalogSourceKind::Po,
            path.to_path_buf(),
            Some(path.to_path_buf()),
            po_file,
            false,
        ))
    }

    pub fn open_pot(path: impl AsRef<Path>, locale: Option<String>) -> Result<Self> {
        let path = path.as_ref();
        let mut po_file = read_po(path)?;
        if let Some(locale) = locale.clone().filter(|value| !value.trim().is_empty()) {
            po_file.metadata.insert("Language".into(), locale);
        }

        Ok(Self::from_po_file(
            CatalogSourceKind::Pot,
            path.to_path_buf(),
            None,
            po_file,
            false,
        ))
    }

    pub fn open_pot_with_existing_po(
        pot_path: impl AsRef<Path>,
        po_path: impl AsRef<Path>,
        locale: Option<String>,
    ) -> Result<Self> {
        let pot_path = pot_path.as_ref();
        let po_path = po_path.as_ref();
        let pot = read_po(pot_path)?;
        let mut po_file = read_po(po_path)?;
        po_file.merge(pot);

        if let Some(locale) = locale.clone().filter(|value| !value.trim().is_empty()) {
            po_file.metadata.insert("Language".into(), locale);
        }

        Ok(Self::from_po_file(
            CatalogSourceKind::Pot,
            pot_path.to_path_buf(),
            Some(po_path.to_path_buf()),
            po_file,
            false,
        ))
    }

    pub fn session(&self) -> &CatalogSession {
        &self.session
    }

    pub fn session_mut(&mut self) -> &mut CatalogSession {
        &mut self.session
    }

    pub fn set_locale(&mut self, locale: impl Into<String>) {
        let locale = locale.into();
        let trimmed = locale.trim().to_string();
        let normalized = (!trimmed.is_empty()).then_some(trimmed);

        self.session.locale = normalized.clone();
        self.po_file
            .metadata
            .insert("Language".into(), normalized.clone().unwrap_or_default());
        self.refresh();
        self.session.dirty = true;
    }

    pub fn update_translation(
        &mut self,
        key: &EntryKey,
        payload: TranslationPayload,
        mark_fuzzy: bool,
    ) -> Result<Vec<crate::ValidationWarning>> {
        let index = self.find_entry_index(key)?;
        let warnings = validate_translation(&self.session.entries[index], &payload);
        let entry = self
            .po_file
            .entries
            .iter_mut()
            .find(|entry| key_for_entry(entry) == *key)
            .ok_or(CatalogError::EntryNotFound)?;

        entry.msgstr = Some(payload.singular);
        entry.msgstr_plural = payload.plurals;

        if mark_fuzzy {
            ensure_flag(entry, "fuzzy");
        }

        self.refresh_entry(key);
        self.session.dirty = true;

        Ok(warnings)
    }

    pub fn set_fuzzy(&mut self, key: &EntryKey, fuzzy: bool) -> Result<()> {
        let entry = self
            .po_file
            .entries
            .iter_mut()
            .find(|entry| key_for_entry(entry) == *key)
            .ok_or(CatalogError::EntryNotFound)?;

        if fuzzy {
            ensure_flag(entry, "fuzzy");
        } else {
            entry.flags.retain(|flag| flag != "fuzzy");
        }

        self.refresh_entry(key);
        self.session.dirty = true;

        Ok(())
    }

    pub fn merge_template(&mut self, pot_path: impl AsRef<Path>) -> Result<()> {
        let pot = read_po(pot_path.as_ref())?;
        self.po_file.merge(pot);
        self.refresh();
        self.session.dirty = true;
        Ok(())
    }

    pub fn save_po(&mut self, path: Option<PathBuf>, auto_compile_mo: bool) -> Result<SaveOutcome> {
        let po_path = if let Some(path) = path {
            path
        } else {
            self.session
                .po_path
                .clone()
                .ok_or(CatalogError::MissingPoPath)?
        };

        self.po_file.save(&po_path.to_string_lossy());
        repair_po_comments(&po_path);
        self.session.po_path = Some(po_path.clone());
        self.session.dirty = false;

        let mo_path = if auto_compile_mo {
            let path = po_path.with_extension("mo");
            self.po_file.save_as_mofile(&path.to_string_lossy());
            self.session.last_compiled_mo = Some(path.clone());
            Some(path)
        } else {
            None
        };

        Ok(SaveOutcome { po_path, mo_path })
    }

    pub fn compile_mo(&mut self, path: Option<PathBuf>) -> Result<PathBuf> {
        let mo_path = if let Some(path) = path {
            path
        } else {
            self.session
                .po_path
                .clone()
                .ok_or(CatalogError::MissingPoPath)?
                .with_extension("mo")
        };

        self.po_file.save_as_mofile(&mo_path.to_string_lossy());
        self.session.last_compiled_mo = Some(mo_path.clone());
        Ok(mo_path)
    }

    fn from_po_file(
        source_kind: CatalogSourceKind,
        source_path: PathBuf,
        po_path: Option<PathBuf>,
        po_file: POFile,
        dirty: bool,
    ) -> Self {
        let mut document = Self {
            session: CatalogSession {
                source_kind,
                source_path,
                po_path,
                locale: None,
                dirty,
                last_compiled_mo: None,
                header: CatalogHeader {
                    raw_header: None,
                    metadata_is_fuzzy: false,
                    metadata: BTreeMap::new(),
                    locale: None,
                    plural_forms: None,
                },
                stats: CatalogStats {
                    total: 0,
                    translated: 0,
                    untranslated: 0,
                    fuzzy: 0,
                    obsolete: 0,
                    warnings: 0,
                },
                entries: Vec::new(),
            },
            po_file,
        };

        document.refresh();
        document
    }

    fn refresh(&mut self) {
        let locale = self.po_file.metadata.get("Language").cloned();
        let plural_forms = self.po_file.metadata.get("Plural-Forms").cloned();
        let metadata = self
            .po_file
            .metadata
            .iter()
            .map(|(key, value)| (key.clone(), value.clone()))
            .collect::<BTreeMap<_, _>>();

        let entries = self
            .po_file
            .entries
            .iter()
            .map(entry_to_model)
            .collect::<Vec<_>>();

        let stats = build_stats(&entries);

        self.session.locale = locale.clone();
        self.session.header = CatalogHeader {
            raw_header: self.po_file.header.clone(),
            metadata_is_fuzzy: self.po_file.metadata_is_fuzzy,
            metadata,
            locale,
            plural_forms,
        };
        self.session.entries = entries;
        self.session.stats = stats;
    }

    fn refresh_entry(&mut self, key: &EntryKey) {
        let Some(po_entry) = self
            .po_file
            .entries
            .iter()
            .find(|e| key_for_entry(e) == *key)
        else {
            return;
        };
        let updated = entry_to_model(po_entry);

        if let Some(existing) = self.session.entries.iter_mut().find(|e| e.key == *key) {
            decrement_stats(&mut self.session.stats, existing);
            *existing = updated;
            increment_stats(&mut self.session.stats, existing);
        }
    }

    fn find_entry_index(&self, key: &EntryKey) -> Result<usize> {
        self.session
            .entries
            .iter()
            .position(|entry| &entry.key == key)
            .ok_or(CatalogError::EntryNotFound)
    }
}

fn read_po(path: &Path) -> Result<POFile> {
    pofile(path.to_string_lossy().as_ref()).map_err(|error| CatalogError::Parse(error.to_string()))
}

fn key_for_entry(entry: &POEntry) -> EntryKey {
    EntryKey {
        msgid: entry.msgid.clone(),
        msgctxt: entry.msgctxt.clone(),
    }
}

fn entry_to_model(entry: &POEntry) -> CatalogEntry {
    let key = key_for_entry(entry);
    let payload = TranslationPayload {
        singular: entry.msgstr.clone().unwrap_or_default(),
        plurals: entry.msgstr_plural.clone(),
    };

    let mut model = CatalogEntry {
        key,
        msgid: entry.msgid.clone(),
        msgid_plural: entry.msgid_plural.clone(),
        msgctxt: entry.msgctxt.clone(),
        msgstr: entry.msgstr.clone().unwrap_or_default(),
        msgstr_plural: entry.msgstr_plural.clone(),
        extracted_comment: entry.comment.clone().unwrap_or_default(),
        translator_comment: entry.tcomment.clone().unwrap_or_default(),
        references: entry
            .occurrences
            .iter()
            .map(|(path, line)| format!("{path}:{line}"))
            .collect(),
        flags: entry.flags.clone(),
        previous_msgid: entry.previous_msgid.clone(),
        previous_msgid_plural: entry.previous_msgid_plural.clone(),
        previous_msgctxt: entry.previous_msgctxt.clone(),
        obsolete: entry.obsolete,
        status: EntryStatus::Untranslated,
        warnings: Vec::new(),
    };

    model.warnings = validate_translation(&model, &payload);
    model.status = classify_entry(&model);
    model
}

fn classify_entry(entry: &CatalogEntry) -> EntryStatus {
    if entry.obsolete {
        return EntryStatus::Obsolete;
    }

    if entry.flags.iter().any(|flag| flag == "fuzzy") {
        return EntryStatus::Fuzzy;
    }

    let translated = !entry.msgstr.trim().is_empty()
        || entry
            .msgstr_plural
            .iter()
            .any(|plural| !plural.trim().is_empty());

    if translated {
        EntryStatus::Translated
    } else {
        EntryStatus::Untranslated
    }
}

fn build_stats(entries: &[CatalogEntry]) -> CatalogStats {
    let mut stats = CatalogStats {
        total: entries.len(),
        translated: 0,
        untranslated: 0,
        fuzzy: 0,
        obsolete: 0,
        warnings: 0,
    };

    for entry in entries {
        match entry.status {
            EntryStatus::Translated => stats.translated += 1,
            EntryStatus::Untranslated => stats.untranslated += 1,
            EntryStatus::Fuzzy => stats.fuzzy += 1,
            EntryStatus::Obsolete => stats.obsolete += 1,
        }

        if !entry.warnings.is_empty() {
            stats.warnings += 1;
        }
    }

    stats
}

fn increment_stats(stats: &mut CatalogStats, entry: &CatalogEntry) {
    match entry.status {
        EntryStatus::Translated => stats.translated += 1,
        EntryStatus::Untranslated => stats.untranslated += 1,
        EntryStatus::Fuzzy => stats.fuzzy += 1,
        EntryStatus::Obsolete => stats.obsolete += 1,
    }
    if !entry.warnings.is_empty() {
        stats.warnings += 1;
    }
}

fn decrement_stats(stats: &mut CatalogStats, entry: &CatalogEntry) {
    match entry.status {
        EntryStatus::Translated => stats.translated = stats.translated.saturating_sub(1),
        EntryStatus::Untranslated => stats.untranslated = stats.untranslated.saturating_sub(1),
        EntryStatus::Fuzzy => stats.fuzzy = stats.fuzzy.saturating_sub(1),
        EntryStatus::Obsolete => stats.obsolete = stats.obsolete.saturating_sub(1),
    }
    if !entry.warnings.is_empty() {
        stats.warnings = stats.warnings.saturating_sub(1);
    }
}

/// Fix rspolib bugs:
/// 1. Multi-line extracted comments lose their `#. ` prefix
/// 2. Reference lines (`#:`) get duplicated on each save, growing exponentially
fn repair_po_comments(path: &Path) {
    let Ok(content) = std::fs::read_to_string(path) else {
        return;
    };

    let mut repaired = String::with_capacity(content.len());
    let mut changed = false;

    for line in content.lines() {
        // Fix 1: bare lines that lost their comment prefix
        if !line.is_empty()
            && !line.starts_with('#')
            && !line.starts_with('"')
            && !line.starts_with("msg")
            && !line.starts_with('\t')
        {
            repaired.push_str("#. ");
            repaired.push_str(line);
            changed = true;
        }
        // Fix 2: deduplicate reference lines that grew from repeated saves
        else if line.starts_with("#:") && line.len() > 1000 {
            let refs_part = &line[2..];
            let mut seen = std::collections::HashSet::new();
            let mut deduped = String::from("#:");
            for token in refs_part.split_whitespace() {
                if seen.insert(token) {
                    deduped.push(' ');
                    deduped.push_str(token);
                }
            }
            if deduped.len() < line.len() {
                changed = true;
            }
            repaired.push_str(&deduped);
        } else {
            repaired.push_str(line);
        }
        repaired.push('\n');
    }

    if changed {
        let _ = std::fs::write(path, repaired);
    }
}

fn ensure_flag(entry: &mut POEntry, flag: &str) {
    if !entry.flags.iter().any(|existing| existing == flag) {
        entry.flags.push(flag.to_string());
    }
}
