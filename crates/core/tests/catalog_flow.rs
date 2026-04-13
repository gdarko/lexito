use std::path::PathBuf;

use lexito_core::{validate_translation, CatalogDocument, EntryStatus, TranslationPayload};
use tempfile::tempdir;

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name)
}

#[test]
fn merges_template_into_existing_po() {
    let document = CatalogDocument::open_pot_with_existing_po(
        fixture_path("messages.pot"),
        fixture_path("fr.po"),
        Some("fr".to_string()),
    )
    .expect("template should merge");

    assert_eq!(document.session().stats.total, 2);
    assert_eq!(document.session().locale.as_deref(), Some("fr"));

    let greeting = document
        .session()
        .entries
        .iter()
        .find(|entry| entry.msgid == "Hello %s")
        .expect("greeting entry");

    assert_eq!(greeting.msgstr, "Bonjour %s");
}

#[test]
fn updates_translation_and_saves_po_and_mo() {
    let tempdir = tempdir().expect("tempdir");
    let mut document = CatalogDocument::open(fixture_path("messages.pot"), Some("fr".to_string()))
        .expect("template should open");
    let key = document.session().entries[0].key.clone();

    let warnings = document
        .update_translation(
            &key,
            TranslationPayload {
                singular: "Bonjour %s".to_string(),
                plurals: Vec::new(),
            },
            true,
        )
        .expect("translation should update");

    assert!(warnings.is_empty());
    assert_eq!(document.session().entries[0].status, EntryStatus::Fuzzy);

    let po_path = tempdir.path().join("fr.po");
    let outcome = document
        .save_po(Some(po_path.clone()), true)
        .expect("save should succeed");

    assert_eq!(outcome.po_path, po_path);
    assert!(outcome.mo_path.expect("auto compiled mo").exists());
}

#[test]
fn validates_placeholder_mismatch() {
    let document = CatalogDocument::open(fixture_path("messages.pot"), Some("fr".to_string()))
        .expect("template should open");
    let entry = &document.session().entries[0];

    let warnings = validate_translation(
        entry,
        &TranslationPayload {
            singular: "Bonjour".to_string(),
            plurals: Vec::new(),
        },
    );

    assert!(!warnings.is_empty());
}

#[test]
fn project_create_and_add_language() {
    let tempdir = tempdir().expect("tempdir");

    // Use tempdir as HOME so projects_root() goes there
    std::env::set_var("HOME", tempdir.path().to_str().unwrap());

    let pot_path = fixture_path("messages.pot");
    let (mut project, dir) = lexito_core::Project::create("TestApp", &pot_path).unwrap();

    assert_eq!(project.project.name, "TestApp");
    assert!(dir.join("source.pot").exists());
    assert!(dir.join("project.toml").exists());

    // Add a language
    project.add_language("fr", &dir).unwrap();
    assert_eq!(project.languages.len(), 1);

    let po_path = project.po_path("fr", &dir).unwrap();
    assert!(po_path.exists(), "PO file should exist at {:?}", po_path);

    // Open the .po and check it works
    let doc = lexito_core::CatalogDocument::open_po(&po_path).unwrap();
    let session = doc.session();
    assert!(session.stats.total > 0, "Should have entries");

    // Load stats
    let stats = project.load_language_stats(&dir);
    assert_eq!(stats.len(), 1);
    assert_eq!(stats[0].0, "fr");

    // Reload project from disk
    let loaded = lexito_core::Project::load(&dir).unwrap();
    assert_eq!(loaded.languages.len(), 1);
    assert_eq!(loaded.languages[0].locale, "fr");
}
