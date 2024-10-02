use std::{collections::HashMap, path::PathBuf};

use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct Env {
    pub slack_app_token: String,
    pub slack_bot_token: String,
    pub dirs_mapping: DirsMapping,
}

impl Env {
    pub fn new() -> Result<Self, envy::Error> {
        envy::from_env()
    }
}

#[derive(Debug, Clone)]
pub struct DirsMapping(pub HashMap<String, PathBuf>);

impl<'de> serde::Deserialize<'de> for DirsMapping {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let raw = String::deserialize(deserializer)?;
        let mut result = HashMap::new();
        for entry in raw.split(',') {
            let (key, value) = entry.split_once(':').ok_or_else(|| {
                serde::de::Error::custom("Invalid entry, expected value separated with =")
            })?;
            result.insert(key.to_owned(), PathBuf::from(value));
        }
        Ok(DirsMapping(result))
    }
}
