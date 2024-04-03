use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

pub type Result<T> = std::result::Result<T, AppDataError>;

#[derive(Debug)]
pub enum AppDataError {
    BrokenConfig,
    InvalidConfigPath,
    Io(std::io::Error),
}

impl From<std::io::Error> for AppDataError {
    fn from(value: std::io::Error) -> Self {
        AppDataError::Io(value)
    }
}

#[derive(Clone)]
pub struct AppData {
    change_flag : bool,
    raw : AppDataRaw,
}


#[derive(Clone, Serialize, Deserialize)]
pub struct AppDataRaw {
    config_path: Option<PathBuf>,
}

impl AppData {
    pub fn read() -> Result<Self> {
        let appdata_path = Self::appdata_path();
        if appdata_path.exists() {
            let appdata_toml = std::fs::read_to_string(appdata_path)?;
            let appdata_raw = toml::from_str::<AppDataRaw>(&appdata_toml).map_err(|err| {
                eprintln!("{err:?}");
                AppDataError::BrokenConfig
            })?;
            Ok(AppData{
                change_flag : false,
                raw : appdata_raw,
            })
        } else {
            Ok(Self::default())
        }
    }

    pub fn set_config_path(&mut self, path: Option<PathBuf>) -> Result<()> {
        let new_config_path = match path {
            Some(path) => Some(std::fs::canonicalize(path)?),
            None => None,
        };
        if let Some(config_path) = new_config_path.clone() {
            if config_path.is_dir() {
                return Err(AppDataError::InvalidConfigPath)
            }
        }
        if new_config_path != self.raw.config_path {
            self.raw.config_path = new_config_path;
            self.change_flag = true;
        }
        Ok(())
    }

    pub fn get_config_path(&self) -> Option<&PathBuf> {
        self.raw.config_path.as_ref()
    }

    fn appdata_path() -> PathBuf {
        let mut appdata_path = dirs::home_dir().expect("No home directory avaiable on the OS");
        appdata_path.push(".canzero");
        appdata_path.push("canzero.toml");
        appdata_path
    }

    fn rec_create_directories(dir: &Path) -> Result<()> {
        if !dir.exists() {
            if let Some(parent) = dir.parent() {
                Self::rec_create_directories(parent)?;
            }
            std::fs::create_dir(dir)?;
        }
        Ok(())
    }
}

impl Drop for AppData {
    fn drop(&mut self) {
        if self.change_flag {
            let appdata_path = Self::appdata_path();
            if let Some(parent) = appdata_path.parent() {
                Self::rec_create_directories(parent)
                    .expect(&format!("Failed to create config directories {parent:?}"));
            }
            let appdata_toml = toml::to_string_pretty(&self.raw).unwrap();
            std::fs::write(appdata_path.clone(), &appdata_toml)
                .expect(&format!("Failed to write to {appdata_path:?}"));
        }
    }
}

impl Default for AppData {
    fn default() -> Self {
        Self {
            change_flag: false,
            raw : AppDataRaw {
                config_path : None,
            }
        }
    }
}
