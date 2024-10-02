use std::path::PathBuf;

use anyhow::Result;
use url::Url;

pub async fn export(path: &PathBuf) -> Result<Url> {
    Ok("https://www.google.com".to_string().parse()?)
}
