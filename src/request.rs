use audiocloud_lib::*;
use reqwest::blocking::Client;
use std::fs::{self};

use crate::helpers;

pub async fn check_connection(ip: String) -> bool {
    let client = Client::new();
    let response = client.get(ip).send();
    match response {
        Ok(val) => match val.text() {
            Ok(_) => {
                return true;
            }
            Err(_) => return false,
        },
        Err(_) => return false,
    }
}
pub async fn get_result(params: SearchParams, path: String) -> SearchResult {
    let client = Client::new();
    let response = client
        .post(path + "search")
        .json(&params)
        .send()
        .expect("Couldnt do reqwest")
        .text()
        .unwrap();

    let out: SearchResult = serde_json::from_str(&response).expect("Couldnt parse response");
    out
}
pub async fn get_editor_audio(
    sample: Sample,
    server_url: String,
    file_path: String,
) -> (Sample, String) {
    let tempaudio_path = "Editor.wav";
    let client = Client::new();
    let file_path_web = file_path.replace("#", "%23").replace(" ", "%20");
    let url = server_url + "samples/" + &file_path_web;
    let response = client
        .get(url)
        .send()
        .expect("Couldnt send file get request");
    let body = response.bytes().expect("body invalid");
    let _ = std::fs::write(tempaudio_path, &body);
    (sample, String::from(tempaudio_path))
}

pub async fn get_temp_audio(server_url: String, file_path: String) -> String {
    let tempaudio_path = "Tempaudio.wav";
    let client = Client::new();
    let file_path_web = file_path.replace("#", "%23").replace(" ", "%20");
    let url = server_url + "samples/" + &file_path_web;
    let response = client
        .get(url)
        .send()
        .expect("Couldnt send file get request");
    let body = response.bytes().expect("body invalid");
    let _ = std::fs::write(tempaudio_path, &body);
    String::from(tempaudio_path)
}

pub async fn get_packs_meta(server_url: String) -> Vec<PackInfo> {
    let client = Client::new();
    let url = server_url + "packs";
    let response = client
        .get(url)
        .send()
        .expect("Couldnt perform request")
        .text()
        .expect("couldnt get text from response");
    print!("{}", response);
    let out: Vec<PackInfo> = serde_json::from_str(&response).expect("Couldn't convert json");
    out
}

pub async fn dl_sample(server_url: String, file_path: String) -> String {
    if !std::path::Path::new("cached/").exists() {
        let _ = fs::create_dir("cached/");
    }
    let folder_path = "cached/".to_string();
    let path = folder_path + &helpers::hash_sample(&file_path.replace(".wav", "")) + ".wav";
    println!("{}", path);
    let client = Client::new();
    let file_path_web = file_path.replace("#", "%23").replace(" ", "%20");
    let url = server_url + "samples/" + &file_path_web;
    let response = client
        .get(url)
        .send()
        .expect("Couldnt send file get request");
    let body = response.bytes().expect("body invalid");
    let _ = std::fs::write(path, &body);
    file_path
}
