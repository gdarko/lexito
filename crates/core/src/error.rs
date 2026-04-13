use std::path::PathBuf;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum CatalogError {
    #[error("unsupported catalog extension for {0}")]
    UnsupportedSource(PathBuf),
    #[error("failed to parse gettext catalog: {0}")]
    Parse(String),
    #[error("entry not found")]
    EntryNotFound,
    #[error("catalog requires a PO destination path before saving")]
    MissingPoPath,
}

pub type Result<T> = std::result::Result<T, CatalogError>;
