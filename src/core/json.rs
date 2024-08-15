use std::fs::File;
use std::io::Write;
use std::path::Path;

use crate::state::{self, AppSettings};

use log;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

const SETTINGS_PATH: &str = "./data/settings.json";

pub async fn load_settings() -> Option<state::AppSettings> {
    let mut contents = String::new();

    match tokio::fs::File::open(SETTINGS_PATH).await {
        Ok(mut f) => {
            let _ = f.read_to_string(&mut contents).await;

            let settings: state::AppSettings = match serde_json::from_str(&contents) {
                Ok(s) => s,
                Err(e) => {
                    log::error!("Error parsing settings file: {}", e);
                    return None;
                }
            };

            Some(settings)

        },
        Err(e) => {
            log::error!("Error opening settings file: {}", e);

            None
        },
    }
}

pub async fn save_settings(settings: state::AppSettings) -> Result<(), std::io::Error> {
    let data = serde_json::to_string_pretty(&settings).unwrap();

    match tokio::fs::File::create(SETTINGS_PATH).await {
        Ok(mut file) => {
            file.write_all(data.as_bytes()).await?;
        }
        Err(e) => {
            log::error!("Error creating settings file: {}", e);
            return Err(e);
        }
    }

    Ok(())
}

pub fn check_exists() -> bool {
    Path::new(SETTINGS_PATH).exists()
}

pub fn create_file() -> Result<(), std::io::Error> {
    let settings = AppSettings {
        ..Default::default()
    };

    let data = serde_json::to_string_pretty(&settings).unwrap();

    match File::create(SETTINGS_PATH) {
        Ok(mut file) => {
            file.write_all(data.as_bytes())?;
        }
        Err(e) => {
            log::error!("Error creating settings file: {}", e);
            return Err(e);
        }
    }

    Ok(())
}
