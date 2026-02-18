# Self-Update Feature

## Completion Date
2026-02-18

## Summary
Added one-click auto-update from GitHub releases. The app checks for updates on startup, shows a non-blocking banner on ModeSelect when a new version is available, and provides a full update screen with download progress and restart functionality.

## Implementation
- Background update check via GitHub API `/releases/latest`
- Version comparison (semver-like, supports `v0.3`, `v0.3.1`, `1.2.3`)
- Download with progress bar to staging path (`%APPDATA%/rust-rdp/rust-rdp-update.exe`)
- Windows self-replacement via `.bat` script (waits for PID exit, copies, relaunches, self-deletes)
- Cleanup of leftover staging files on next launch

## Files Added
- `src/updater.rs` — update check, download, apply logic, version parsing
- `src/ui/update.rs` — update screen UI (Available, Downloading, ReadyToInstall, Applying, Error states)

## Files Modified
- `Cargo.toml` — added `json` feature to reqwest, bumped version to 0.3.1
- `src/main.rs` — added `mod updater`
- `src/ui/mod.rs` — added `pub mod update`
- `src/ui/mode_select.rs` — added `UpdateClicked` message, `available_update` field, update banner
- `src/app.rs` — wired Update screen, messages, subscription, update check on startup

## Tests
- 9 new tests (parse_version variants, is_newer comparisons, staging path, cleanup safety)
- Total: 54 tests passing

## Follow-up
- GitHub release workflow to auto-build and publish `rust-rdp.exe`
- Consider rate-limit handling for GitHub API (60 req/hr unauthenticated)
