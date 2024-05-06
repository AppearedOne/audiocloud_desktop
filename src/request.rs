use audiocloud_lib::*;
use reqwest::Client;
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
