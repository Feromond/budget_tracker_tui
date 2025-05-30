use serde::{Deserialize, Serialize};
use std::fs::{create_dir_all, File};
use std::io::{Error, ErrorKind, Read, Write};
use std::path::PathBuf;

const APP_CONFIG_SUBDIR: &str = "BudgetTracker";
const CONFIG_FILE_NAME: &str = "config.json";

#[derive(Serialize, Deserialize, Debug, Default)]
pub(crate) struct AppSettings {
    pub(crate) data_file_path: Option<String>,
    pub(crate) target_budget: Option<f64>,
    pub(crate) necessary_expenses_percentage: Option<u8>,
    pub(crate) discretionary_expenses_percentage: Option<u8>,
    pub(crate) saving_or_investment_percentage: Option<u8>,
    pub(super) tax_setaside_percentage: Option<u8>,
}

fn get_config_file_path() -> Result<PathBuf, Error> {
    match dirs::config_dir() {
        Some(mut path) => {
            path.push(APP_CONFIG_SUBDIR);
            create_dir_all(&path)?; // Ensure the directory exists
            path.push(CONFIG_FILE_NAME);
            Ok(path)
        }
        None => Err(Error::new(
            ErrorKind::NotFound,
            "Could not find user config directory",
        )),
    }
}

pub(crate) fn load_settings() -> Result<AppSettings, Error> {
    let config_path = get_config_file_path()?;

    if !config_path.exists() {
        return Ok(AppSettings::default());
    }

    let mut file = File::open(config_path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    serde_json::from_str(&contents).map_err(|e| {
        Error::new(
            ErrorKind::InvalidData,
            format!("Failed to parse config file: {}", e),
        )
    })
}

pub(crate) fn save_settings(settings: &AppSettings) -> Result<(), Error> {
    let config_path = get_config_file_path()?;

    let contents = serde_json::to_string_pretty(settings)
        .map_err(|e| Error::other(format!("Failed to serialize settings: {}", e)))?;

    let mut file = File::create(config_path)?;
    file.write_all(contents.as_bytes())?;

    Ok(())
}
