use anyhow::*;
use serde_derive::*;
#[derive(Serialize, Deserialize, Debug)]
pub struct Settings {
    pub theme: String,
    pub favourite_samples: Vec<String>,
}

pub fn load_from_file(path: String) -> Settings {
    todo!();
}

pub fn save_to_file(path: String, settings: &Settings) -> Result<(), anyhow::Error> {
    todo!();
}
