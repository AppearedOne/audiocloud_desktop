use audiocloud_lib::Sample;
use iced::Theme;
use serde_derive::*;
use std::fs;
use std::path::Path;

use crate::helpers::{self, hash_sample};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Settings {
    pub theme: String,
    pub server_url: String,
    pub max_results: i32,
    pub favourite_samples: Vec<Sample>,
    pub dl_samples_hash: Vec<String>,
}
pub async fn load_from_file(path: &str) -> Settings {
    if !Path::new(path).exists() {
        return Settings {
            max_results: 50,
            server_url: "http://127.0.0.1:4040/".to_string(),
            theme: Theme::Dark.to_string(),
            favourite_samples: vec![],
            dl_samples_hash: vec![],
        };
    }
    let filecontent = fs::read_to_string(path).expect("Couldn't read file");
    let settings: Settings = serde_json::from_str(&filecontent).expect("Couldnt parse file");
    settings
}

pub async fn save_to_file(settings: Settings, path: &str) {
    let content = serde_json::to_string_pretty(&settings).unwrap();
    let _ = fs::write(path, content);
}

impl Settings {
    pub fn is_favourite(&self, sample: &Sample) -> bool {
        for s in &self.favourite_samples {
            if s.path == sample.path {
                return true;
            }
        }
        false
    }
    pub fn add_favourite(&mut self, sample: Sample) {
        self.favourite_samples.push(sample);
    }
    pub fn rem_favourite(&mut self, sample_id: &str) {
        for i in 0..self.favourite_samples.len() {
            if self.favourite_samples[i].path.eq(sample_id) {
                self.favourite_samples.remove(i);
                break;
            }
        }
    }
    pub fn is_downloaded(&self, path: &str) -> bool {
        for entry in &self.dl_samples_hash {
            if entry == &hash_sample(&path.replace(".wav", "")) {
                return true;
            }
        }
        false
    }
    pub fn add_dl_entry(&mut self, path: &str) {
        self.dl_samples_hash.push(helpers::hash_sample(path));
        println!("added dl entry: {}", helpers::hash_sample(path));
    }
}
