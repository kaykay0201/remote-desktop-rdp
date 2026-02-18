# Self-Update Feature

## Objective
Add one-click auto-update from GitHub releases with background update check, banner notification, download progress, and bat-based restart.

## Files to Create
- `src/updater.rs` — update check, download, apply logic
- `src/ui/update.rs` — update screen UI

## Files to Modify
- `Cargo.toml` — add `json` feature to reqwest, bump version
- `src/main.rs` — add `mod updater`
- `src/ui/mod.rs` — add `pub mod update`
- `src/ui/mode_select.rs` — add update banner
- `src/app.rs` — wire everything together

## Risks
- Windows bat script for self-replacement
- GitHub API rate limiting (unauthenticated: 60 req/hr)
