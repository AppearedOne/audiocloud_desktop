use std::fs;
use xxhash_rust::xxh3::xxh3_64;

pub fn remove_brackets(input: &str) -> String {
    let mut result = String::new();
    let mut inside_brackets = false;

    for c in input.chars() {
        if c == '[' {
            inside_brackets = true;
        } else if c == ']' {
            inside_brackets = false;
        } else if !inside_brackets {
            result.push(c);
        }
    }

    result
}

pub fn hash_sample(path: &str) -> String {
    xxh3_64(path.replace(".wav", "").as_bytes()).to_string()
}

// deletes all files in "cached" and returns an empty vector to be used as list of downloaded files
pub async fn clear_cached() -> Vec<String> {
    let path = "cached/";
    fs::remove_dir_all(path).unwrap();
    fs::create_dir(path).unwrap();
    vec![]
}
