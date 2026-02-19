use std::path::{Path, PathBuf};

use serde::Deserialize;
use sha2::{Digest, Sha256};
use tokio::sync::mpsc;
use tracing::info;

#[derive(Debug, Clone)]
pub struct ReleaseInfo {
    pub version: String,
    pub download_url: String,
    pub checksum_url: Option<String>,
    pub body: String,
}

#[derive(Debug, Clone)]
pub enum UpdateProgress {
    Started { total_bytes: u64 },
    Progress { downloaded: u64, total: u64 },
    Verifying,
    Finished(PathBuf),
    Error(String),
}

#[derive(Deserialize)]
struct GitHubRelease {
    tag_name: String,
    body: Option<String>,
    assets: Vec<GitHubAsset>,
}

#[derive(Deserialize)]
struct GitHubAsset {
    name: String,
    browser_download_url: String,
}

pub fn parse_version(tag: &str) -> Option<(u32, u32, u32)> {
    let tag = tag.strip_prefix('v').unwrap_or(tag);
    let parts: Vec<&str> = tag.split('.').collect();
    if parts.is_empty() {
        return None;
    }
    let major = parts[0].parse::<u32>().ok()?;
    let minor = parts.get(1).and_then(|s| s.parse::<u32>().ok()).unwrap_or(0);
    let patch = parts.get(2).and_then(|s| s.parse::<u32>().ok()).unwrap_or(0);
    Some((major, minor, patch))
}

pub fn is_newer(remote_tag: &str, current: &str) -> bool {
    match (parse_version(remote_tag), parse_version(current)) {
        (Some(remote), Some(curr)) => remote > curr,
        _ => false,
    }
}

pub async fn check_for_update() -> Result<Option<ReleaseInfo>, String> {
    let client = reqwest::Client::builder()
        .user_agent("rust-rdp")
        .build()
        .map_err(|e| format!("Failed to build HTTP client: {e}"))?;

    let release: GitHubRelease = client
        .get("https://api.github.com/repos/kaykay0201/remote-desktop-rdp/releases/latest")
        .send()
        .await
        .map_err(|e| format!("Failed to fetch release: {e}"))?
        .json()
        .await
        .map_err(|e| format!("Failed to parse release: {e}"))?;

    let current = env!("CARGO_PKG_VERSION");
    if !is_newer(&release.tag_name, current) {
        return Ok(None);
    }

    let asset = release
        .assets
        .iter()
        .find(|a| a.name == "rust-rdp.exe")
        .ok_or_else(|| "No rust-rdp.exe asset found in release".to_string())?;

    let checksum_url = release
        .assets
        .iter()
        .find(|a| a.name == "rust-rdp.exe.sha256")
        .map(|a| a.browser_download_url.clone());

    Ok(Some(ReleaseInfo {
        version: release.tag_name,
        download_url: asset.browser_download_url.clone(),
        checksum_url,
        body: release.body.unwrap_or_default(),
    }))
}

pub async fn download_update(
    url: String,
    progress_tx: mpsc::Sender<UpdateProgress>,
) -> Result<PathBuf, String> {
    use futures::StreamExt;
    use tokio::io::AsyncWriteExt;

    let dir = crate::cloudflared::managed_dir();
    tokio::fs::create_dir_all(&dir)
        .await
        .map_err(|e| format!("Failed to create directory: {e}"))?;

    let dest = staging_exe_path();

    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Download request failed: {e}"))?;

    if !response.status().is_success() {
        return Err(format!("Download failed with status: {}", response.status()));
    }

    let total_bytes = response.content_length().unwrap_or(0);
    let _ = progress_tx
        .send(UpdateProgress::Started { total_bytes })
        .await;

    let mut stream = response.bytes_stream();
    let mut file = tokio::fs::File::create(&dest)
        .await
        .map_err(|e| format!("Failed to create file: {e}"))?;

    let mut downloaded: u64 = 0;

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| format!("Download stream error: {e}"))?;
        file.write_all(&chunk)
            .await
            .map_err(|e| format!("Failed to write chunk: {e}"))?;
        downloaded += chunk.len() as u64;
        let _ = progress_tx
            .send(UpdateProgress::Progress {
                downloaded,
                total: total_bytes,
            })
            .await;
    }

    file.flush()
        .await
        .map_err(|e| format!("Failed to flush file: {e}"))?;

    info!("Update downloaded to {}", dest.display());
    let _ = progress_tx
        .send(UpdateProgress::Finished(dest.clone()))
        .await;

    Ok(dest)
}

pub fn compute_sha256(path: &Path) -> Result<String, String> {
    let bytes = std::fs::read(path).map_err(|e| format!("Failed to read file for hashing: {e}"))?;
    let hash = Sha256::digest(&bytes);
    Ok(format!("{:x}", hash))
}

pub async fn verify_checksum(exe_path: &Path, checksum_url: &str) -> Result<(), String> {
    let client = reqwest::Client::builder()
        .user_agent("rust-rdp")
        .build()
        .map_err(|e| format!("Failed to build HTTP client: {e}"))?;

    let response = client
        .get(checksum_url)
        .send()
        .await
        .map_err(|e| format!("Failed to download checksum: {e}"))?;

    if !response.status().is_success() {
        return Err(format!(
            "Checksum download failed with status: {}",
            response.status()
        ));
    }

    let checksum_text = response
        .text()
        .await
        .map_err(|e| format!("Failed to read checksum: {e}"))?;

    let expected_hash = checksum_text
        .split_whitespace()
        .next()
        .ok_or_else(|| "Empty checksum file".to_string())?
        .to_lowercase();

    let actual_hash = compute_sha256(exe_path)?;

    if actual_hash != expected_hash {
        return Err(format!(
            "Checksum mismatch: expected {expected_hash}, got {actual_hash}"
        ));
    }

    info!("SHA256 verification passed");
    Ok(())
}

pub fn apply_update(new_exe_path: &Path) -> Result<(), String> {
    let dir = crate::cloudflared::managed_dir();
    let backup_path = dir.join("rust-rdp-backup.exe");

    let current_exe =
        std::env::current_exe().map_err(|e| format!("Failed to get current exe: {e}"))?;

    std::fs::copy(&current_exe, &backup_path)
        .map_err(|e| format!("Failed to create backup: {e}"))?;
    info!("Backed up current exe to {}", backup_path.display());

    self_replace::self_replace(new_exe_path)
        .map_err(|e| format!("Self-replace failed: {e}"))?;
    info!("Self-replace succeeded");

    let _ = std::fs::remove_file(new_exe_path);

    let current_exe =
        std::env::current_exe().map_err(|e| format!("Failed to get new exe path: {e}"))?;
    std::process::Command::new(current_exe)
        .spawn()
        .map_err(|e| format!("Failed to relaunch: {e}"))?;

    Ok(())
}

fn update_marker_path() -> PathBuf {
    crate::cloudflared::managed_dir().join(".update-ok")
}

fn backup_exe_path() -> PathBuf {
    crate::cloudflared::managed_dir().join("rust-rdp-backup.exe")
}

pub fn check_post_update_health() {
    let marker = update_marker_path();
    let backup = backup_exe_path();

    if backup.exists() && !marker.exists() {
        let _ = std::fs::write(&marker, "ok");
        info!("Post-update: marker created, backup preserved for one session");
    } else if backup.exists() && marker.exists() {
        let _ = std::fs::remove_file(&backup);
        let _ = std::fs::remove_file(&marker);
        info!("Post-update: backup and marker cleaned up");
    }
}

pub fn staging_exe_path() -> PathBuf {
    crate::cloudflared::managed_dir().join("rust-rdp-update.exe")
}


pub fn cleanup_old_update() {
    let staging = staging_exe_path();
    if staging.exists() {
        let _ = std::fs::remove_file(&staging);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_version_full() {
        assert_eq!(parse_version("v0.3.1"), Some((0, 3, 1)));
    }

    #[test]
    fn parse_version_short() {
        assert_eq!(parse_version("v0.3"), Some((0, 3, 0)));
    }

    #[test]
    fn parse_version_no_prefix() {
        assert_eq!(parse_version("1.2.3"), Some((1, 2, 3)));
    }

    #[test]
    fn parse_version_invalid() {
        assert_eq!(parse_version("invalid"), None);
    }

    #[test]
    fn is_newer_true() {
        assert!(is_newer("v0.4.0", "0.3.1"));
    }

    #[test]
    fn is_newer_false_same() {
        assert!(!is_newer("v0.3.1", "0.3.1"));
    }

    #[test]
    fn is_newer_false_older() {
        assert!(!is_newer("v0.2.0", "0.3.1"));
    }

    #[test]
    fn staging_path_correct() {
        let path = staging_exe_path();
        assert_eq!(path.file_name().unwrap(), "rust-rdp-update.exe");
    }

    #[test]
    fn cleanup_no_panic() {
        cleanup_old_update();
    }

    #[test]
    fn compute_sha256_works() {
        let dir = std::env::temp_dir().join("rust-rdp-test-sha256");
        let _ = std::fs::create_dir_all(&dir);
        let test_file = dir.join("test.bin");
        std::fs::write(&test_file, b"hello world").unwrap();
        let hash = compute_sha256(&test_file).unwrap();
        assert_eq!(
            hash,
            "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
        );
        let _ = std::fs::remove_file(&test_file);
        let _ = std::fs::remove_dir(&dir);
    }

    #[test]
    fn backup_path_correct() {
        let path = backup_exe_path();
        assert_eq!(path.file_name().unwrap(), "rust-rdp-backup.exe");
    }

    #[test]
    fn marker_path_correct() {
        let path = update_marker_path();
        assert_eq!(path.file_name().unwrap(), ".update-ok");
    }

    #[test]
    fn health_check_no_panic() {
        check_post_update_health();
    }
}
