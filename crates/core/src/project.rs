use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::{CatalogError, Result};
use crate::model::CatalogStats;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub project: ProjectMeta,
    pub source: ProjectSource,
    #[serde(default)]
    pub languages: Vec<ProjectLanguage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectMeta {
    pub name: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectSource {
    pub pot_file: String,
    pub original_path: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectLanguage {
    pub locale: String,
    pub po_file: String,
}

impl Project {
    pub fn create(name: &str, pot_source_path: &Path) -> Result<(Self, PathBuf)> {
        let project_dir = projects_root().join(name);
        if project_dir.exists() {
            return Err(CatalogError::Parse(format!(
                "Project directory already exists: {}",
                project_dir.display()
            )));
        }

        fs::create_dir_all(&project_dir)
            .map_err(|e| CatalogError::Parse(format!("Could not create project dir: {e}")))?;

        let pot_filename = "source.pot";
        let dest_pot = project_dir.join(pot_filename);
        fs::copy(pot_source_path, &dest_pot)
            .map_err(|e| CatalogError::Parse(format!("Could not copy .pot file: {e}")))?;

        let now = chrono_now();

        let project = Project {
            project: ProjectMeta {
                name: name.to_string(),
                created_at: now,
            },
            source: ProjectSource {
                pot_file: pot_filename.to_string(),
                original_path: pot_source_path.to_path_buf(),
            },
            languages: Vec::new(),
        };

        project.save(&project_dir)?;
        Ok((project, project_dir))
    }

    pub fn load(project_dir: &Path) -> Result<Self> {
        let toml_path = project_dir.join("project.toml");
        let content = fs::read_to_string(&toml_path)
            .map_err(|e| CatalogError::Parse(format!("Could not read project file: {e}")))?;
        toml::from_str(&content)
            .map_err(|e| CatalogError::Parse(format!("Invalid project file: {e}")))
    }

    pub fn save(&self, project_dir: &Path) -> Result<()> {
        let toml_path = project_dir.join("project.toml");
        let content = toml::to_string_pretty(self)
            .map_err(|e| CatalogError::Parse(format!("Could not serialize project: {e}")))?;
        fs::write(toml_path, content)
            .map_err(|e| CatalogError::Parse(format!("Could not write project file: {e}")))
    }

    pub fn add_language(&mut self, locale: &str, project_dir: &Path) -> Result<()> {
        let po_filename = format!("{locale}.po");
        let pot_path = self.pot_path(project_dir);
        let po_path = project_dir.join(&po_filename);

        // Allow re-adding if the .po file is missing (stale entry from failed add)
        let already_exists = self.languages.iter().any(|l| l.locale == locale);
        if already_exists && po_path.exists() {
            return Err(CatalogError::Parse(format!(
                "Language '{locale}' already exists in this project"
            )));
        }

        // Copy the .pot file as .po, then patch the Language header directly in
        // the file text. We avoid round-tripping through rspolib's serializer
        // because it corrupts multi-line extracted comments.
        fs::copy(&pot_path, &po_path)
            .map_err(|e| CatalogError::Parse(format!("Could not copy .pot to .po: {e}")))?;
        let content = fs::read_to_string(&po_path)
            .map_err(|e| CatalogError::Parse(format!("Could not read .po: {e}")))?;
        let patched = content.replace("\"Language: \\n\"", &format!("\"Language: {locale}\\n\""));
        fs::write(&po_path, patched)
            .map_err(|e| CatalogError::Parse(format!("Could not write .po: {e}")))?;

        if !already_exists {
            self.languages.push(ProjectLanguage {
                locale: locale.to_string(),
                po_file: po_filename,
            });
        }

        self.save(project_dir)?;
        Ok(())
    }

    pub fn remove_language(&mut self, locale: &str, project_dir: &Path) -> Result<()> {
        if let Some(lang) = self.languages.iter().find(|l| l.locale == locale) {
            let po_path = project_dir.join(&lang.po_file);
            if po_path.exists() {
                let _ = fs::remove_file(&po_path);
            }
            // Also remove compiled .mo if present
            let mo_path = po_path.with_extension("mo");
            if mo_path.exists() {
                let _ = fs::remove_file(&mo_path);
            }
        }
        self.languages.retain(|l| l.locale != locale);
        self.save(project_dir)?;
        Ok(())
    }

    pub fn pot_path(&self, project_dir: &Path) -> PathBuf {
        project_dir.join(&self.source.pot_file)
    }

    pub fn po_path(&self, locale: &str, project_dir: &Path) -> Option<PathBuf> {
        self.languages
            .iter()
            .find(|l| l.locale == locale)
            .map(|l| project_dir.join(&l.po_file))
    }

    pub fn load_language_stats(&self, project_dir: &Path) -> Vec<(String, CatalogStats)> {
        self.languages
            .iter()
            .filter_map(|lang| {
                let po_path = project_dir.join(&lang.po_file);
                quick_po_stats(&po_path)
                    .ok()
                    .map(|stats| (lang.locale.clone(), stats))
            })
            .collect()
    }
}

pub fn projects_root() -> PathBuf {
    let home = std::env::var("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."));
    home.join("Lexito")
}

pub fn list_projects() -> Result<Vec<(String, PathBuf)>> {
    let root = projects_root();
    if !root.exists() {
        return Ok(Vec::new());
    }

    let mut projects = Vec::new();
    let entries =
        fs::read_dir(&root).map_err(|e| CatalogError::Parse(format!("Cannot read dir: {e}")))?;

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() && path.join("project.toml").exists() {
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                projects.push((name.to_string(), path));
            }
        }
    }

    projects.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(projects)
}

/// Fast stats by scanning the raw .po text instead of full rspolib parse + validation.
fn quick_po_stats(po_path: &Path) -> Result<CatalogStats> {
    let content = fs::read_to_string(po_path)
        .map_err(|e| CatalogError::Parse(format!("Could not read .po: {e}")))?;

    let mut total = 0usize;
    let mut translated = 0usize;
    let mut fuzzy = 0usize;
    let mut obsolete = 0usize;
    let mut is_fuzzy = false;
    let mut is_obsolete = false;
    let mut in_msgstr = false;
    let mut msgstr_has_content = false;
    let mut seen_msgid = false;

    for line in content.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with("#,") && trimmed.contains("fuzzy") {
            is_fuzzy = true;
        }
        if trimmed.starts_with("#~") {
            is_obsolete = true;
        }

        if trimmed.starts_with("msgid ") && !trimmed.starts_with("msgid_plural") {
            // Commit previous entry
            if seen_msgid {
                total += 1;
                if is_obsolete {
                    obsolete += 1;
                } else if is_fuzzy {
                    fuzzy += 1;
                } else if msgstr_has_content {
                    translated += 1;
                }
            }
            // Reset for new entry
            seen_msgid = true;
            in_msgstr = false;
            msgstr_has_content = false;
            is_fuzzy = false;
            is_obsolete = false;
        }

        if trimmed.starts_with("msgstr ") || trimmed.starts_with("msgstr[") {
            in_msgstr = true;
            let value = trimmed
                .split_once(' ')
                .map(|x| x.1)
                .unwrap_or("")
                .trim_matches('"');
            if !value.is_empty() {
                msgstr_has_content = true;
            }
        } else if in_msgstr && trimmed.starts_with('"') {
            let value = trimmed.trim_matches('"');
            if !value.is_empty() {
                msgstr_has_content = true;
            }
        } else if !trimmed.is_empty()
            && !trimmed.starts_with('#')
            && !trimmed.starts_with("msgid")
            && !trimmed.starts_with("msgctxt")
        {
            in_msgstr = false;
        }
    }

    // Commit last entry
    if seen_msgid {
        total += 1;
        if is_obsolete {
            obsolete += 1;
        } else if is_fuzzy {
            fuzzy += 1;
        } else if msgstr_has_content {
            translated += 1;
        }
    }

    // Subtract 1 for the header entry (empty msgid)
    if total > 0 {
        total -= 1;
        // Header is always "translated" (has msgstr), undo that count
        translated = translated.saturating_sub(1);
    }

    let untranslated = total.saturating_sub(translated + fuzzy + obsolete);

    Ok(CatalogStats {
        total,
        translated,
        untranslated,
        fuzzy,
        obsolete,
        warnings: 0, // Skip validation for dashboard stats
    })
}

fn chrono_now() -> String {
    // Simple ISO 8601 timestamp without chrono dependency
    use std::time::SystemTime;
    let duration = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default();
    let secs = duration.as_secs();
    // Format as a simple timestamp
    format!("{secs}")
}
