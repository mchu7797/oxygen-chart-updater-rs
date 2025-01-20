use config::{Config, ConfigError};
use serde::Deserialize;
use std::path::{Path, PathBuf};

#[derive(Deserialize)]
pub struct ConfigValues {
    pub database_url: String,
    pub upload_directory: PathBuf,
    pub update_directory: PathBuf,
    pub delete_directory: PathBuf,
}

pub fn get_config() -> Result<ConfigValues, ConfigError> {
    const CONFIG_FILE_NAME: &str = "config.json";

    if !Path::new(CONFIG_FILE_NAME).exists() {
        return Err(ConfigError::NotFound(CONFIG_FILE_NAME.to_string()));
    }

    let config = Config::builder()
        .add_source(config::File::with_name(CONFIG_FILE_NAME))
        .build()?;

    let config_values: ConfigValues = config.try_deserialize()?;

    if config_values.database_url.is_empty() {
        return Err(ConfigError::NotFound("database url (mssql)".to_string()));
    }

    if !config_values.upload_directory.exists() {
        return Err(ConfigError::NotFound("upload directory".to_string()));
    }

    if !config_values.update_directory.exists() {
        return Err(ConfigError::NotFound("update directory".to_string()));
    }

    if !config_values.delete_directory.exists() {
        return Err(ConfigError::NotFound("delete directory".to_string()));
    }

    Ok(config_values)
}
