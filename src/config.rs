use std::{collections::BTreeMap, fs, io::Write, path::PathBuf};

use directories::{BaseDirs, ProjectDirs};
use serde::{Deserialize, Serialize};
use tempfile::NamedTempFile;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub enum Language {
    #[default]
    Spanish,
    English,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    pub install_root: PathBuf,
    pub selected: BTreeMap<String, String>,
    pub bash_enabled: bool,
    pub bash_block: Option<String>,
    #[serde(default)]
    pub language: Language,
}

impl Default for Config {
    fn default() -> Self {
        let install_root = BaseDirs::new()
            .map(|dirs| dirs.home_dir().join("Tools").join("pswitcher"))
            .unwrap_or_else(|| PathBuf::from("~/Tools/pswitcher"));

        Self {
            install_root,
            selected: BTreeMap::new(),
            bash_enabled: false,
            bash_block: None,
            language: Language::Spanish,
        }
    }
}

pub fn load() -> Result<Config, String> {
    let path = config_path()?;
    match fs::read(&path) {
        Ok(bytes) => serde_json::from_slice(&bytes).map_err(|error| error.to_string()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(Config::default()),
        Err(error) => Err(format!("No se pudo leer {}: {error}", path.display())),
    }
}

pub fn save(config: &Config) -> Result<(), String> {
    let path = config_path()?;
    let parent = path.parent().ok_or("Ruta de configuración inválida")?;
    fs::create_dir_all(parent).map_err(|error| error.to_string())?;

    let mut file = NamedTempFile::new_in(parent).map_err(|error| error.to_string())?;
    serde_json::to_writer_pretty(&mut file, config).map_err(|error| error.to_string())?;
    file.write_all(b"\n").map_err(|error| error.to_string())?;
    file.persist(&path).map_err(|error| error.to_string())?;
    Ok(())
}

fn config_path() -> Result<PathBuf, String> {
    ProjectDirs::from("", "", "pswitcher")
        .map(|dirs| dirs.config_dir().join("config.json"))
        .ok_or_else(|| "No se pudo resolver el directorio de configuración del usuario".into())
}
