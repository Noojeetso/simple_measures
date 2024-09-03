use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::time::Duration;

use anyhow::Result;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct TargetDescription {
    pub filename: String,
    pub description: String,
    pub max_size_number: usize,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct PackMeasuresDescription<GenArgT>
where
    GenArgT: std::fmt::Display,
{
    pub description: String,
    pub filename: String,
    pub sizes: Vec<GenArgT>,
    pub x_label: String,
    pub y_label: String,
    pub iterations_amount: u64,
    pub threshold: Duration,
    pub target_descriptions: Vec<TargetDescription>,
}

impl<GenArgT> PackMeasuresDescription<GenArgT>
where
    GenArgT: std::fmt::Display + serde::ser::Serialize,
{
    pub fn write(&self, dir_path: &Path) -> Result<()> {
        if !dir_path.is_dir() {
            std::fs::create_dir_all(dir_path)?;
        }
        let path = dir_path.join(&PathBuf::from_str("description.json")?);
        let data = match serde_json::to_string_pretty(self) {
            Ok(config) => config,
            Err(e) => {
                eprintln!("Error: {}\n", e);
                return Err(e.into());
            }
        };
        // println!("path: {}", path.as_os_str().to_str().unwrap());
        match fs::write(path, data) {
            Ok(()) => Ok(()),
            Err(e) => Err(e.into()),
        }
    }
}

impl<GenArgT> PackMeasuresDescription<GenArgT>
where
    GenArgT: std::fmt::Display + serde::de::DeserializeOwned,
{
    pub fn read(path: &Path) -> Result<Self> {
        let data = match fs::read_to_string(path) {
            Ok(data) => data,
            Err(e) => {
                eprintln!("Ошибка чтения файла конфигурации");
                eprintln!("Error: {}\n", e);
                return Err(e.into());
            }
        };
        let description: PackMeasuresDescription<GenArgT> = match serde_json::from_str(&data) {
            Ok(config) => config,
            Err(e) => {
                eprintln!("Повреждена структура файла конфигурации");
                eprintln!("Error: {}\n", e);
                return Err(e.into());
            }
        };
        Ok(description)
    }
}
