use std::collections::BTreeMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CatalogSourceKind {
    Po,
    Pot,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EntryKey {
    pub msgid: String,
    pub msgctxt: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EntryStatus {
    Translated,
    Untranslated,
    Fuzzy,
    Obsolete,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ValidationCode {
    PlaceholderMismatch,
    TagMismatch,
    PluralMismatch,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidationWarning {
    pub code: ValidationCode,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TranslationPayload {
    pub singular: String,
    pub plurals: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CatalogEntry {
    pub key: EntryKey,
    pub msgid: String,
    pub msgid_plural: Option<String>,
    pub msgctxt: Option<String>,
    pub msgstr: String,
    pub msgstr_plural: Vec<String>,
    pub extracted_comment: String,
    pub translator_comment: String,
    pub references: Vec<String>,
    pub flags: Vec<String>,
    pub previous_msgid: Option<String>,
    pub previous_msgid_plural: Option<String>,
    pub previous_msgctxt: Option<String>,
    pub obsolete: bool,
    pub status: EntryStatus,
    pub warnings: Vec<ValidationWarning>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CatalogHeader {
    pub raw_header: Option<String>,
    pub metadata_is_fuzzy: bool,
    pub metadata: BTreeMap<String, String>,
    pub locale: Option<String>,
    pub plural_forms: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CatalogStats {
    pub total: usize,
    pub translated: usize,
    pub untranslated: usize,
    pub fuzzy: usize,
    pub obsolete: usize,
    pub warnings: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CatalogSession {
    pub source_kind: CatalogSourceKind,
    pub source_path: PathBuf,
    pub po_path: Option<PathBuf>,
    pub locale: Option<String>,
    pub dirty: bool,
    pub last_compiled_mo: Option<PathBuf>,
    pub header: CatalogHeader,
    pub stats: CatalogStats,
    pub entries: Vec<CatalogEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SaveOutcome {
    pub po_path: PathBuf,
    pub mo_path: Option<PathBuf>,
}
