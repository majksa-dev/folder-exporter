mod zip;

use anyhow::Result;
use std::path::Path;
use url::Url;

pub async fn export(path: &Path) -> Result<Url> {
    let archive_file = zip::zip_folder(path)?;
    Ok(format!("https://www.google.com/{}", archive_file.to_str().unwrap()).parse()?)
}
