use std::path::PathBuf;

use lexito_ai::{AppSettings, BatchProgressEvent, ModelInfo, ProviderType, ThemePreference};
use lexito_core::{CatalogDocument, CatalogEntry, CatalogStats, EntryKey, EntryStatus, Project};
use uuid::Uuid;

use crate::locales::Locale;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppScreen {
    ProjectList,
    ProjectDashboard,
    Workspace,
    Settings,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntryFilter {
    All,
    Untranslated,
    Fuzzy,
    Warnings,
    Obsolete,
}

impl EntryFilter {
    pub fn matches(self, entry: &CatalogEntry) -> bool {
        match self {
            EntryFilter::All => true,
            EntryFilter::Untranslated => entry.status == EntryStatus::Untranslated,
            EntryFilter::Fuzzy => entry.status == EntryStatus::Fuzzy,
            EntryFilter::Warnings => !entry.warnings.is_empty(),
            EntryFilter::Obsolete => entry.status == EntryStatus::Obsolete,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            EntryFilter::All => "All",
            EntryFilter::Untranslated => "Untranslated",
            EntryFilter::Fuzzy => "Fuzzy",
            EntryFilter::Warnings => "Warnings",
            EntryFilter::Obsolete => "Obsolete",
        }
    }
}

#[derive(Debug, Clone)]
pub struct SaveCatalogResult {
    pub document: CatalogDocument,
    pub message: String,
}

#[derive(Debug, Clone)]
pub struct SettingsSaved {
    pub settings: AppSettings,
}

#[derive(Debug, Clone)]
pub struct SingleTranslationFinished {
    pub key: EntryKey,
    pub response: lexito_ai::TranslationResponse,
}

#[derive(Debug, Clone)]
#[allow(clippy::type_complexity)]
pub enum Message {
    // Navigation
    GoToProjects,
    GoToDashboard,
    GoToSettings,
    SaveAndGo,
    ConfirmDiscard,
    CancelNavigation,

    // Project list
    OpenProject(PathBuf),
    ProjectLoaded(Result<(Project, PathBuf, Vec<(String, CatalogStats)>), String>),
    DeleteProject(PathBuf),

    // Create project
    NewProjectNameChanged(String),
    PickPotFile,
    PotFilePicked(Option<PathBuf>),
    CreateProject,
    ProjectCreated(Result<(Project, PathBuf), String>),

    // Project dashboard
    AddLanguage(Locale),
    LanguageAdded(Result<(Project, String), String>),
    RemoveLanguage(String),
    OpenLanguage(String),
    LanguageOpened(Result<(CatalogDocument, String), String>),

    // Project actions
    OpenProjectFolder,

    // Keyboard
    KeyboardEvent(iced::keyboard::Event),
    SelectNextEntry,
    SelectPrevEntry,

    // Workspace (catalog editing)
    SavePressed,
    SavePathPicked(Option<PathBuf>),
    CatalogSaved(Result<SaveCatalogResult, String>),
    SelectEntry(EntryKey),
    FilterSelected(EntryFilter),
    SingularChanged(String),
    PluralChanged(usize, String),
    ApplyLocalEdit,
    ToggleSelectedFuzzy,
    TranslateSelectedPressed,
    SingleTranslationFinished(Result<SingleTranslationFinished, String>),
    BatchTranslateUntranslated,
    BatchTranslateFuzzy,
    ApproveAllFuzzy,
    BatchProgress(BatchProgressEvent),
    CancelBatch,
    SpinnerTick,

    // Provider management
    SelectProvider(Uuid),
    EditProvider(Uuid),
    AddProvider,
    RemoveProvider(Uuid),
    CancelEditProvider,
    DraftNameChanged(String),
    DraftProviderTypeChanged(ProviderType),
    DraftApiKeyChanged(String),
    DraftModelSelected(ModelInfo),
    FetchModels,
    ModelsFetched(Result<Vec<ModelInfo>, String>),
    SaveProvider,
    ProviderSaved(Result<SettingsSaved, String>),

    // Translation preferences
    ThemePreferenceChanged(ThemePreference),
    TemperatureChanged(String),
    TimeoutChanged(String),
    ConcurrencyChanged(String),
    DefaultLocaleChanged(Locale),
    SystemPromptChanged(String),
    AutoCompileToggled(bool),
    SaveSettingsPressed,
    SettingsSaved(Result<SettingsSaved, String>),
}
