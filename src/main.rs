use std::{env, fs::{self, File}, io::BufReader, path::PathBuf, process::exit};

use clap::{Parser, Subcommand};
use reqwest::blocking::Client;
use serde::{Deserialize};

mod api;
mod config;

/* This is a CLI tool for interacting with syncthing with the focus on: 
 * adding shared folders, managing connections, getting statistics and an overview of the sync */

#[derive(Parser)]
#[command(name = "syncth", about = "A CLI tool for interacting with Syncthing")]
struct Cli{
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    List {},
    Add {
        #[arg(default_value_t = current_dir())]
        path: String,

        #[arg(short = 't', long, default_value_t = api::SendType::SendOnly)]
        type_: api::SendType,
    },
    Share {
        #[arg(short, long = "folder")]
        folder_label: String,

        #[arg(short, long = "device")]
        device_label: String,
    },
    Unshare {
        #[arg(short, long = "folder")]
        folder_label: String,

        #[arg(short, long = "device")]
        device_label: String,

    },
    ConnectFolder {},
}

fn current_dir() -> String {
    env::current_dir()
        .unwrap_or_else(|_| { eprintln!("Could not get current directory"); exit(1) } )
        .to_string_lossy()
        .to_string()
}


fn get_folder_devices<'a>(config: &'a config::Configuration, client: &Client, folder: &config::Folder) -> Vec<&'a config::Device> {
    let own_id = api::get_own_id(client, config.gui.api_key.as_str()).unwrap();
    config.devices.iter().filter_map(|device| {
        folder.devices.iter().find_map(|folder_device| {
            if folder_device.id == device.id && device.id != own_id {
                Some(device)
            } else {
                None
            }
        })
    }).collect()
}

fn list_folders(config: &config::Configuration, client: &Client) -> Result<(), Box<dyn std::error::Error>> {
    let label_width = 20;
    let path_width = 50;
    let device_width = 20;

    println!("{:<label_width$} {:<path_width$} {:<device_width$}", "Label", "Path", "Shared Devices");

    for folder in &config.folders {
        let devices = get_folder_devices(config, client, folder);
        println!("{:<label_width$} {:<path_width$} {:<device_width$}",
            folder.label, folder.path,
            devices
            .iter()
            .filter(|device| device.name.is_some())
            .map(|device| device.name.as_ref().unwrap().as_str())
            .collect::<Vec<&str>>().join(", "));
    }

    Ok(())
}

fn add_folder(config: &config::Configuration, client: &Client, folder_path: &PathBuf, type_: &api::SendType) -> Result<(), Box<dyn std::error::Error>> {
    api::post_add_folder(client, &config.gui.api_key, folder_path, type_)?;
    Ok(())
}

fn share_folder(config: &config::Configuration, client: &Client, folder_id: &str, device_id: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut folder_json = api::get_folder(client, &config.gui.api_key, folder_id)?;

    match folder_json.get_mut("devices") {
        Some(devices) => {
            devices.as_array_mut().unwrap().push(serde_json::json!({
                "deviceID": device_id,
                "encryptionPassword": "",
                "introducedBy": ""
            }));

            api::put_folder(client, &config.gui.api_key, folder_id, &folder_json)?;
        },
        None => {
            eprintln!("Could not set devices in folder config");
            exit(1);
        }
    }
    
    Ok(())
}

fn unshare_folder(config: &config::Configuration, client: &Client, folder_id: &str, device_id: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut folder_json = api::get_folder(client, &config.gui.api_key, folder_id)?;


    match folder_json.get_mut("devices") {
        Some(devices) => {
            devices
                .as_array_mut()
                .unwrap()
                .retain(|device| device["deviceID"].as_str().unwrap() != device_id);

            api::put_folder(client, &config.gui.api_key, folder_id, &folder_json)?;
        },
        None => {
            eprintln!("Could not set devices in folder config");
            exit(1);
        }
    }
    
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let config = config::Configuration::parse()?;
    let client = Client::new();

    match cli.command {
        Commands::List{  } => {
            list_folders(&config, &client)
        },
        Commands::Add{ path, type_} => {
            add_folder(&config, &client, &fs::canonicalize(&path)?, &type_)
        },
        Commands::Share{ folder_label, device_label  } => {
            let folder_id = config.file_id(&folder_label)
                .unwrap_or_else(|| { eprintln!("Could not find folder with label: {}", folder_label); exit(1) });
            let device_id = config.device_id(&device_label)
                .unwrap_or_else(|| { eprintln!("Could not find device with label: {}", device_label); exit(1) });
            share_folder(&config, &client, folder_id, device_id)
        }
        Commands::Unshare { folder_label, device_label } => {
            let folder_id = config.file_id(&folder_label)
                .unwrap_or_else(|| { eprintln!("Could not find folder with label: {}", folder_label); exit(1) });
            let device_id = config.device_id(&device_label)
                .unwrap_or_else(|| { eprintln!("Could not find device with label: {}", device_label); exit(1) });
            unshare_folder(&config, &client, folder_id, device_id)
        }
        _ => {
            todo!();
        }
    }
}
