mod drive;
mod zip;

use anyhow::{anyhow, Result};
use std::path::Path;
use url::Url;

pub fn authorize() -> Result<()> {
    drive::authorize(Path::new("./secret/config.json"))
        .map_err(|e| anyhow!("unable to upload the archive to Google Drive: {}", e))
}

pub async fn export(path: &Path, parent_id: String) -> Result<Url> {
    let archive_file = zip::zip_folder(path)?;
    tokio::task::spawn_blocking(move || {
        drive::upload(&archive_file, parent_id)
            .map_err(|e| anyhow!("unable to upload the archive to Google Drive: {}", e))
    })
    .await
    .expect("Task panicked")
}
