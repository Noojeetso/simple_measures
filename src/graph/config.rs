use anyhow::Result;
use fs_err as fs;

use std::fmt;
use std::path::Path;

#[derive(Debug, serde::Deserialize)]
pub enum GraphOutputType {
    NONE,
    SVG,
    PDF,
    PNG,
}

impl fmt::Display for GraphOutputType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let output: &str;
        match self {
            Self::NONE => output = "",
            Self::SVG => output = "svg",
            Self::PDF => output = "pdf",
            Self::PNG => output = "png",
        }
        write!(f, "{}", output)
    }
}

#[derive(Debug, serde::Deserialize)]
pub struct GraphConfig {
    pub output_type: GraphOutputType,
    pub save_temp_files: bool,
    pub x_start: u32,
    pub x_end: u32,
    pub x_scale: f64,
    pub log_x: bool,
    pub y_start: u32,
    pub y_end: u32,
    pub y_scale: f64,
    pub log_y: bool,
}

impl GraphConfig {
    pub fn read(config_path: &Path) -> Result<Self> {
        let data = match fs::read_to_string(config_path) {
            Ok(data) => data,
            Err(e) => {
                eprintln!("Ошибка чтения файла конфигурации");
                eprintln!("Error: {}\n", e);
                return Err(e.into());
            }
        };
        let config: GraphConfig = match serde_json::from_str(&data) {
            Ok(config) => config,
            Err(e) => {
                eprintln!("Повреждена структура файла конфигурации");
                eprintln!("Error: {}\n", e);
                return Err(e.into());
            }
        };
        Ok(config)
    }
}

// pub fn read_configs(dir_path: &Path) -> Result<Vec<GraphConfig>> {
//     let configs = vec![];

//     let algorithm_paths = std::fs::read_dir(dir_path)?;
//     for algorithm_path_result in algorithm_paths {
//         let algorithm_path = algorithm_path_result
//             .expect("Ошибка чтения пути файла")
//             .path();
//         let algo_name = algorithm_path.to_str().unwrap().split("/").last().unwrap();
//     }

//     Ok(configs)
// }
