use std::path::Path;

use drive_v3::{
    objects::{File, UploadType},
    Credentials, Drive, Result,
};
use url::Url;

const SCOPES: [&str; 2] = [
    "https://www.googleapis.com/auth/drive.metadata.readonly",
    "https://www.googleapis.com/auth/drive.file",
];

const STORAGE_PATH: &str = "./secret/credentials.json";

pub fn authorize(secrets: &Path) -> Result<()> {
    let credentials = Credentials::from_client_secrets_file(secrets, &SCOPES)?;
    credentials.store(STORAGE_PATH)?;
    Ok(())
}

pub fn upload(path: &Path, parent_id: String) -> Result<Url> {
    let mut credentials = Credentials::from_file(STORAGE_PATH, &SCOPES)?;
    if !credentials.are_valid() {
        credentials.refresh()?;
        credentials.store(STORAGE_PATH)?;
    }
    let metadata = File {
        name: Some(path.file_name().unwrap().to_string_lossy().to_string()),
        mime_type: Some("application/zip".to_string()),
        parents: Some(vec![parent_id]),
        ..Default::default()
    };

    let drive = Drive::new(&credentials);
    let uploaded_file = drive
        .files
        .create()
        .upload_type(UploadType::Resumable)
        .metadata(&metadata)
        .content_source(path)
        .execute()?;
    Ok(format!(
        "https://drive.google.com/file/d/{}/view?usp=drive_link",
        uploaded_file.id.unwrap()
    )
    .parse()?)
}
