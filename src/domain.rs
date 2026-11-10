use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub enum Tool {
    Php,
    Node,
    Composer,
}

impl Tool {
    pub const ALL: [Self; 3] = [Self::Php, Self::Node, Self::Composer];

    pub fn label(self) -> &'static str {
        match self {
            Self::Php => "PHP",
            Self::Node => "Node.js",
            Self::Composer => "Composer",
        }
    }

    pub fn slug(self) -> &'static str {
        match self {
            Self::Php => "php",
            Self::Node => "node",
            Self::Composer => "composer",
        }
    }

    pub fn executable(self) -> &'static str {
        match self {
            Self::Php => "php",
            Self::Node => "node",
            Self::Composer => "composer.phar",
        }
    }
}

#[derive(Clone, Debug)]
pub struct Release {
    pub tool: Tool,
    pub version: String,
    pub download_url: String,
    pub sha256: String,
    pub min_php: Option<u32>,
}

#[derive(Clone, Debug)]
pub struct InstalledVersion {
    pub tool: Tool,
    pub version: String,
    pub path: PathBuf,
}
