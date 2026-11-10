use crate::domain::{InstalledVersion, Tool};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    fs::{self, OpenOptions},
    io::{self, Write},
    path::PathBuf,
};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Settings {
    pub install_root: String,
    pub selected: BTreeMap<Tool, String>,
    pub bash_enabled: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            install_root: "~/Tools/pswitcher".into(),
            selected: BTreeMap::new(),
            bash_enabled: false,
        }
    }
}

impl Settings {
    pub fn load() -> io::Result<Self> {
        let path = config_path()?;
        match fs::read(path) {
            Ok(bytes) => serde_json::from_slice(&bytes)
                .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error)),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(Self::default()),
            Err(error) => Err(error),
        }
    }

    pub fn save(&self) -> io::Result<()> {
        let path = config_path()?;
        let parent = path
            .parent()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "ruta de configuración inválida"))?;
        fs::create_dir_all(parent)?;
        let bytes = serde_json::to_vec_pretty(self)
            .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
        let temp = parent.join(format!("config.{}.tmp", std::process::id()));
        let mut file = OpenOptions::new().write(true).create_new(true).open(&temp)?;
        let result = (|| {
            file.write_all(&bytes)?;
            file.sync_all()?;
            fs::rename(&temp, path)
        })();
        if result.is_err() {
            let _ = fs::remove_file(temp);
        }
        result
    }

    pub fn resolved_root(&self) -> io::Result<PathBuf> {
        let home = directories::BaseDirs::new()
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "no se encontró el directorio personal"))?
            .home_dir()
            .to_path_buf();
        let expanded = if self.install_root == "~" {
            home
        } else if let Some(rest) = self.install_root.strip_prefix("~/") {
            home.join(rest)
        } else {
            PathBuf::from(&self.install_root)
        };
        if !expanded.is_absolute() || expanded == PathBuf::from("/") {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "la carpeta debe ser una ruta absoluta distinta de /",
            ));
        }
        Ok(expanded)
    }
}

pub fn scan_installed(root: &PathBuf) -> Vec<InstalledVersion> {
    let mut installed = Vec::new();
    for tool in Tool::ALL {
        let folder = root.join(tool.slug());
        let Ok(entries) = fs::read_dir(folder) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let version = entry.file_name().to_string_lossy().into_owned();
            let executable = match tool {
                Tool::Php => path.join("bin/php"),
                Tool::Node => path.join("bin/node"),
                Tool::Composer => path.join("composer.phar"),
            };
            if executable.is_file() {
                installed.push(InstalledVersion { tool, version, path });
            }
        }
    }
    installed.sort_by(|left, right| (left.tool, &left.version).cmp(&(right.tool, &right.version)));
    installed
}

fn config_path() -> io::Result<PathBuf> {
    let dirs = ProjectDirs::from("dev", "Pswitcher", "Pswitcher")
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "no hay directorio de configuración"))?;
    Ok(dirs.config_dir().join("config.json"))
}
