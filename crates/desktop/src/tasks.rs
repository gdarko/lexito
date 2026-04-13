use std::path::PathBuf;

use iced::Task;
use lexito_ai::TranslationRequest;
use lexito_core::{CatalogDocument, CatalogEntry};

use crate::types::{Message, SaveCatalogResult};

pub fn request_for_entry(entry: CatalogEntry, target_locale: String) -> TranslationRequest {
    let mut comments = Vec::new();
    if !entry.extracted_comment.trim().is_empty() {
        comments.push(entry.extracted_comment);
    }
    if !entry.translator_comment.trim().is_empty() {
        comments.push(entry.translator_comment);
    }

    TranslationRequest {
        key: entry.key,
        target_locale,
        msgid: entry.msgid,
        msgid_plural: entry.msgid_plural,
        msgctxt: entry.msgctxt,
        comments,
        references: entry.references,
    }
}

pub fn save_catalog_task(
    document: CatalogDocument,
    path: Option<PathBuf>,
    auto_compile_on_save: bool,
) -> Task<Message> {
    Task::perform(
        async move {
            let mut document = document;
            let outcome = document
                .save_po(path, auto_compile_on_save)
                .map_err(|error| error.to_string())?;
            let mut message = format!("Saved {}", outcome.po_path.display());
            if let Some(mo_path) = outcome.mo_path {
                message.push_str(&format!(" and compiled {}", mo_path.display()));
            }
            Ok(SaveCatalogResult { document, message })
        },
        Message::CatalogSaved,
    )
}
