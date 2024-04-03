use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

pub type Result<T> = std::result::Result<T, AppDataError>;

#[derive(Debug)]
pub enum AppDataError {
    BrokenConfig,
    Io(std::io::Error),
}

impl From<std::io::Error> for AppDataError {
    fn from(value: std::io::Error) -> Self {
        AppDataError::Io(value)
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct AppData {
    #[serde(skip_serializing)]
    change_flag: bool, //defaults to false
    config_path: Option<PathBuf>,
}

impl AppData {
    pub fn read() -> Result<Self> {
        let appdata_path = Self::appdata_path();
        if appdata_path.exists() {
            let appdata_toml = std::fs::read_to_string(appdata_path)?;
            let appdata =
                toml::from_str::<AppData>(&appdata_toml).map_err(|_| AppDataError::BrokenConfig)?;
            Ok(appdata)
        } else {
            Ok(Self::default())
        }
    }

    pub fn set_config_path(&mut self, path: Option<PathBuf>) -> Result<()> {
        let new_config_path = match path {
            Some(path) => Some(std::fs::canonicalize(path)?),
            None => None,
        };
        if new_config_path != self.config_path {
            self.config_path = new_config_path;
            self.change_flag = true;
        }
        Ok(())
    }

    pub fn get_config_path(&self) -> Option<&PathBuf> {
        self.config_path.as_ref()
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
            let appdata_toml = toml::to_string_pretty(self).unwrap();
            std::fs::write(appdata_path.clone(), &appdata_toml)
                .expect(&format!("Failed to write to {appdata_path:?}"));
        }
    }
}

impl Default for AppData {
    fn default() -> Self {
        Self {
            config_path: None,
            change_flag: false,
        }
    }
}
