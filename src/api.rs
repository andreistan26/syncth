use reqwest::blocking::{Client, Response};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{error::Error, path::PathBuf};
use rand::{Rng};
use clap::ValueEnum;

use crate::config::{Configuration, Folder};

#[derive(Debug, Deserialize, Serialize)]
pub struct FileInfoBrowse {
    pub name: String,
    #[serde(rename = "modTime")]
    pub mod_time: String,
    pub size: i64,
    #[serde(rename = "type")]
    pub file_type: String,
    #[serde(default)]
    pub children: Vec<FileInfoBrowse>,
}

#[derive(Debug, Clone, ValueEnum, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum SendType {
    #[clap(name = "receiveonly")]
    ReceiveOnly,
    #[clap(name = "sendonly")]
    SendOnly,
    #[clap(name = "sendreceive")]
    SendReceive,
}

impl ToString for SendType {
    fn to_string(&self) -> String {
        match self {
            SendType::ReceiveOnly => "receiveonly",
            SendType::SendOnly => "sendonly",
            SendType::SendReceive => "sendreceive",
        }.to_string()
    }
}

const DEFAULT_URL: &str = "http://localhost:8384";

pub fn fetch_folder_contents(client: &Client, api_key: &str, folder_id: &str) -> Result<Vec<FileInfoBrowse>, Box<dyn Error>> {
    let url = format!("{}/rest/db/browse?folder={}", DEFAULT_URL, folder_id);
    let response: Vec<FileInfoBrowse> = client
        .get(&url)
        .header("X-API-Key", api_key)
        .send()?
        .json()?;

    Ok(response)
}

fn generate_id() -> String {
    let mut rng = rand::thread_rng();
    let part1: String = (&mut rng).sample_iter(&rand::distr::Alphanumeric).take(5).map(char::from).collect();
    let part2: String = (&mut rng).sample_iter(&rand::distr::Alphanumeric).take(5).map(char::from).collect();
    format!("{}-{}", part1, part2)
}

pub fn post_add_folder(client: &Client, api_key: &str, folder_path: &PathBuf, send_type: &SendType) -> Result<(), Box<dyn Error>> {
    let url = format!("{}/rest/config/folders", DEFAULT_URL);
    let response: Response = client
        .post(&url)
        .header("X-API-Key", api_key)
        .body(serde_json::json!({
            "path": folder_path,
            "id": generate_id(),
            "type": send_type.to_string(),
            "label": PathBuf::from(folder_path).file_name().unwrap().to_str().unwrap()
        }).to_string())
        .send()?;
    
    if response.status().is_success() {
        println!("Folder added successfully");
    } else {
        println!("Failed to add folder: status {}", response.status());
    }

    Ok(())
}

pub fn get_own_id(client: &Client, api_key: &str) -> Result<String, Box<dyn Error>> {
    let url = format!("{}/rest/system/status", DEFAULT_URL);
    let response: serde_json::Value = client
        .get(&url)
        .header("X-API-Key", api_key)
        .send()?
        .json()?;

    Ok(response["myID"].as_str().unwrap().to_string())
}

pub fn get_config(client: &Client, api_key: &str) -> Result<Configuration, Box<dyn Error>> {
    let url = format!("{}/rest/config", DEFAULT_URL);
    let response: serde_json::Value = client
        .get(&url)
        .header("X-API-Key", api_key)
        .send()?
        .json()?;

    serde_json::from_value(response).map_err(|e| e.into())
}

pub fn get_folder(client: &Client, api_key: &str, folder_id: &str) -> Result<serde_json::Value, Box<dyn Error>> {
    let url = format!("{}/rest/config/folders/{}", DEFAULT_URL, folder_id);
    let response = client
        .get(&url)
        .header("X-API-Key", api_key)
        .send()?
        .json::<serde_json::Value>()?;

    assert_eq!(response.get("id").unwrap(), folder_id);
    Ok(response)
}

pub fn put_folder(client: &Client, api_key: &str, folder_id: &str, folder_json: &serde_json::Value) -> Result<(), Box<dyn Error>> {
    let url = format!("{}/rest/config/folders/{}", DEFAULT_URL, folder_id);
    let response: Response = client
        .put(&url)
        .header("X-API-Key", api_key)
        .body(folder_json.to_string())
        .send()?;

    if response.status().is_success() {
        println!("Device added to folder successfully");
    } else {
        println!("Failed to add device to folder: status {}", response.status());
    }

    Ok(())
}
