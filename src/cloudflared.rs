use std::path::PathBuf;

use tokio::sync::mpsc;
use tracing::info;

const DOWNLOAD_URL: &str =
    "https://github.com/cloudflare/cloudflared/releases/latest/download/cloudflared-windows-amd64.exe";

#[derive(Debug, Clone)]
pub enum DownloadProgress {
    Started { total_bytes: u64 },
    Progress { downloaded: u64, total: u64 },
    Finished(PathBuf),
    Error(String),
}

pub fn managed_dir() -> PathBuf {
    if let Some(data_dir) = dirs_next::data_dir() {
        data_dir.join("rust-rdp")
    } else if let Ok(appdata) = std::env::var("APPDATA") {
        PathBuf::from(appdata).join("rust-rdp")
    } else {
        PathBuf::from(".").join("rust-rdp")
    }
}

pub fn managed_exe_path() -> PathBuf {
    managed_dir().join("cloudflared.exe")
}

pub fn cloudflared_path() -> Option<PathBuf> {
    let managed = managed_exe_path();
    if managed.exists() {
        return Some(managed);
    }

    which::which("cloudflared").ok()
}

pub async fn download_cloudflared(
    progress_tx: mpsc::Sender<DownloadProgress>,
) -> Result<PathBuf, String> {
    use futures::StreamExt;

    let dir = managed_dir();
    tokio::fs::create_dir_all(&dir)
        .await
        .map_err(|e| format!("Failed to create directory: {e}"))?;

    let dest = managed_exe_path();

    let client = reqwest::Client::new();
    let response = client
        .get(DOWNLOAD_URL)
        .send()
        .await
        .map_err(|e| format!("Download request failed: {e}"))?;

    if !response.status().is_success() {
        return Err(format!("Download failed with status: {}", response.status()));
    }

    let total_bytes = response.content_length().unwrap_or(0);
    let _ = progress_tx
        .send(DownloadProgress::Started { total_bytes })
        .await;

    let mut stream = response.bytes_stream();
    let mut file = tokio::fs::File::create(&dest)
        .await
        .map_err(|e| format!("Failed to create file: {e}"))?;

    let mut downloaded: u64 = 0;

    use tokio::io::AsyncWriteExt;
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| format!("Download stream error: {e}"))?;
        file.write_all(&chunk)
            .await
            .map_err(|e| format!("Failed to write chunk: {e}"))?;
        downloaded += chunk.len() as u64;
        let _ = progress_tx
            .send(DownloadProgress::Progress {
                downloaded,
                total: total_bytes,
            })
            .await;
    }

    file.flush()
        .await
        .map_err(|e| format!("Failed to flush file: {e}"))?;

    info!("cloudflared downloaded to {}", dest.display());
    let _ = progress_tx
        .send(DownloadProgress::Finished(dest.clone()))
        .await;

    Ok(dest)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn managed_dir_is_not_empty() {
        let dir = managed_dir();
        assert!(!dir.as_os_str().is_empty());
    }

    #[test]
    fn managed_exe_path_ends_with_exe() {
        let path = managed_exe_path();
        assert_eq!(path.file_name().unwrap(), "cloudflared.exe");
    }

    #[test]
    fn managed_exe_is_inside_managed_dir() {
        let dir = managed_dir();
        let exe = managed_exe_path();
        assert!(exe.starts_with(&dir));
    }
}
