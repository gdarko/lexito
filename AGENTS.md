# AGENTS.md

This file provides guidance to coding agents like Claude Code (claude.ai/code), Codex, OpenCode, and others when working with code in this repository.

## Project

Lexito is a Rust desktop gettext translator (macOS/Linux) built with Iced. It provides a GUI for editing `.po`/`.pot` translation files with AI-powered translation via OpenAI-compatible APIs (OpenAI, OpenRouter, Anthropic).

## Build & Development Commands

```bash
cargo test --workspace              # Run all tests
cargo test -p lexito-core           # Test core crate only
cargo test -p lexito-ai             # Test AI crate only
cargo test -p lexito-core -- test_name  # Run a single test
cargo fmt --check                   # Check formatting
cargo fmt                           # Auto-format
cargo clippy                        # Lint
cargo run -p lexito         # Run the desktop app
cargo build --release -p lexito  # Release build
```

CI runs: `fmt --check` → `test --workspace` → `build --release -p lexito` on Ubuntu and macOS.

## Architecture

Three-crate workspace (`crates/`), strict dependency hierarchy: **desktop → ai → core**.

### core — Catalog parsing, validation, project management

- `CatalogDocument` wraps rspolib's `POFile` with a `CatalogSession` that caches stats, entry status, and validation warnings. Stats are updated incrementally via `refresh_entry()` on every mutation — never recomputed from scratch.
- `validation.rs` checks placeholder (`%s`, `{}`, etc.), HTML/XML tag, and plural form consistency between source and translation.
- `project.rs` manages filesystem-based projects under `~/Lexito/{name}/` — each contains `project.toml`, `source.pot`, and per-locale `.po`/`.mo` files. Discovery is via directory scan, no database.
- **rspolib workaround**: `repair_po_comments()` applies regex fixes after every `.po` save to patch rspolib bugs with multi-line extracted comments and reference line duplication.
- Test fixtures live in `crates/core/tests/fixtures/`.

### ai — API client, settings, keychain

- `AiClient` supports OpenAI-compatible (`/chat/completions`) and Anthropic (`/messages`) endpoints. Single-entry and batched translation with streaming progress via `batch_stream()`.
- Batch chunking: 30 entries per API call, configurable concurrency (default 4) via `buffer_unordered()`. Response parsing is flexible — handles raw JSON, objects with named arrays, markdown-fenced JSON, and single objects.
- `SettingsStore` persists `AppSettings` as TOML to platform config dirs (`directories::ProjectDirs`). `SecretStore` stores API keys in the system keychain under service `com.lexito.desktop` (hardcoded — changing it orphans existing keys).
- Depends on core only for `TranslationPayload` and `EntryKey`.

### desktop — Single Iced application (Elm architecture)

- State lives in `LexitoApp`. UI flow: `boot()` → `update(Message)` → `view()` with `subscription()` for keyboard events.
- ~43 `Message` variants grouped by feature: navigation, project ops, workspace editing, batch translation, settings/provider management, keyboard shortcuts.
- **Editing flow**: `SelectEntry` → `sync_editor_from_selection()` → user edits → `ApplyLocalEdit` (Cmd+Enter) → `update_translation()` with validation → `SavePressed` (Cmd+S).
- **Batch flow**: `BatchTranslateUntranslated`/`BatchTranslateFuzzy` → `batch_stream()` spawns task → progress streamed as `BatchProgress` messages → cancellation via dropping `batch_handle`.
- **Provider draft pattern**: Editing a provider loads it into a mutable `ProviderDraft`; on save, it's split back into `ProviderProfile` (persisted as TOML) + API key (persisted to keychain).
- Keyboard shortcuts in workspace: `↓`/`↑` navigate entries, `Cmd+S` save, `Cmd+Enter` apply edit, `Cmd+T` translate selected.
- Theme is Catppuccin Mocha, hardcoded in `theme()`.

## Versioning & Releases

The version lives in two places:

1. **`Cargo.toml` (workspace root), line `version = "…"`** — single source of truth for all three crates (each sub-crate inherits via `version.workspace = true`).
2. **`macos/Info.plist`** — contains `__VERSION__` placeholders. This is **not** manually bumped; `macos/bundle.sh --version <tag>` substitutes the real version at build time.

### How to bump the version

1. Update `version` in the root `Cargo.toml` (e.g. `"0.2.0"`).
2. Commit, then tag: `git tag v0.2.0 && git push origin v0.2.0`.
3. The `release.yml` workflow triggers on the `v*` tag, builds all 4 platform archives (linux x86_64, linux aarch64, macos x86_64, macos aarch64), stamps the tag version into the macOS bundle's Info.plist, and creates a GitHub release with auto-generated changelog.

Do **not** manually edit `Info.plist` version strings — they are overwritten by CI.

### Release artifacts

Archives are named `lexito-{tag}-{os}-{arch}.tar.gz`. macOS archives contain `Lexito.app`; Linux archives contain the `lexito` binary. The install script (`install.sh`) downloads the correct archive for the user's platform.

## Key Details

- Rust 1.94.1 pinned in `rust-toolchain.toml` (with clippy + rustfmt components)
- Shared dependencies defined in workspace `Cargo.toml` `[workspace.dependencies]`
- Linux builds need system GUI libs (see `.github/workflows/ci.yml` for apt packages)
- Locale resolution: per-file `locale_input` overrides `default_locale_input` from settings
- Windows is not supported — CI only runs Ubuntu and macOS
