use audiocloud_lib::*;
use reqwest::Client;
use std::fs::{self};

use crate::error::*;
use crate::helpers;

pub async fn check_connection(ip: String) -> bool {
    let client = Client::new();
    let response = client.get(ip).send().await;
    match response {
        Ok(val) => match val.text().await {
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
        .await
        .expect("Couldnt do reqwest")
        .text()
        .await
        .unwrap();

    let out: SearchResult = serde_json::from_str(&response).expect("Couldnt parse response");
    out
}
pub async fn get_editor_audio(sample: Sample, server_url: String) -> (Sample, String) {
    let file_path = sample.path.clone();
    let tempaudio_path = "editor.wav";
    let client = Client::new();
    let file_path_web = file_path.replace("#", "%23").replace(" ", "%20");
    let url = server_url + "samples/" + &file_path_web;
    let response = client
        .get(url)
        .send()
        .await
        .expect("Couldnt send file get request");
    let body = response.bytes().await.expect("body invalid");
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
        .await
        .expect("Couldnt send file get request");
    let body = response.bytes().await.expect("body invalid");
    let _ = std::fs::write(tempaudio_path, &body);
    String::from(tempaudio_path)
}

pub async fn get_packs_meta(server_url: String) -> Result<Vec<PackInfo>, Error> {
    let client = Client::new();
    let url = server_url + "packs";
    let response_s: reqwest::Response = match client.get(url).send().await {
        Err(_) => return Err(Error::new(ErrorType::Connection)),
        Ok(val) => val,
    };

    let response = match response_s.text().await {
        Err(_) => return Err(Error::new(ErrorType::Parse)),
        Ok(val) => val,
    };
    let out: Vec<PackInfo> = match serde_json::from_str(&response) {
        Err(_) => return Err(Error::new(ErrorType::JSON)),
        Ok(val) => val,
    };
    Ok(out)
}

pub async fn dl_sample(server_url: String, file_path: String) -> String {
    if !std::path::Path::new("cached/").exists() {
        let _ = fs::create_dir("cached/");
    }
    let folder_path = "cached/".to_string();
    let path = folder_path + &helpers::hash_sample(&file_path.replace(".wav", "")) + ".wav";
    println!("Downloadpath: {}", path);
    let client = Client::new();
    let file_path_web = file_path.replace("#", "%23").replace(" ", "%20");
    println!("URLPATH: {}", file_path_web);
    let url = server_url + "samples/" + &file_path_web;
    let response = client
        .get(url)
        .send()
        .await
        .expect("Couldnt send file get request");
    let body = response.bytes().await.expect("body invalid");
    let _ = std::fs::write(path, &body);
    file_path
}

pub async fn nothing() {}
