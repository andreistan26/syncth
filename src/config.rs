use serde::{Deserialize, Serialize};

const SYNCTHING_CFG_PATH: &str = "~/.config/syncthing/config.xml";

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Configuration {
    #[serde(rename = "folder", alias = "folders")]
    pub folders: Vec<Folder>,
    
    #[serde(rename = "device", alias = "devices")]
    pub devices: Vec<Device>,

    #[serde(default)]
    pub gui: Gui,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Folder {
    #[serde(rename = "@id", alias = "id")]
    pub id: String,

    #[serde(rename = "@label", alias = "label")]
    pub label: String,

    #[serde(rename = "@path", alias = "path")]
    pub path: String,

    #[serde(rename = "device", alias = "devices")]
    pub devices: Vec<FolderDevice>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Device {
    #[serde(rename = "@id")]
    pub id: String,

    #[serde(rename = "@name")]
    pub name: Option<String>,
}

// This is found in the folder config
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FolderDevice {
    #[serde(rename = "deviceID", alias = "@id")]
    pub id: String,
}


#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Gui {
    #[serde(rename = "apikey")]
    pub api_key: String,
}

impl Configuration {
    pub fn parse() -> Result<Self, Box<dyn std::error::Error>> {
        let home_path = std::env::var("HOME")?;
        let config_filepath = std::path::PathBuf::from(home_path)
            .join(SYNCTHING_CFG_PATH.strip_prefix("~/").unwrap());

        let reader = std::io::BufReader::new(std::fs::File::open(config_filepath)?);
        let config: Configuration = quick_xml::de::from_reader(reader)?;
        Ok(config)
    }

    pub fn file_id(&self, label: &str) -> Option<&str> {
        self.folders.iter().find_map(|folder| {
            if folder.label == label {
                Some(folder.id.as_str())
            } else {
                None
            }
        })
    }

    pub fn device_id(&self, name: &str) -> Option<&str> {
        self.devices.iter().find_map(|device| {
            if device.name.as_deref() == Some(name) {
                Some(device.id.as_str())
            } else {
                None
            }
        })
    }
}
