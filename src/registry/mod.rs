use std::collections::HashMap;
use std::path::Path;

use tx3_sdk::tii::Protocol;

#[derive(thiserror::Error, Debug)]
pub enum RegistryError {
    #[error("failed to read protocols directory: {0}")]
    ReadDir(#[from] std::io::Error),

    #[error("failed to parse TII file {path}: {source}")]
    Parse {
        path: String,
        source: tx3_sdk::tii::Error,
    },

    #[error("missing protocol.name in {0}")]
    MissingName(String),
}

pub struct TiiRegistry {
    protocols: HashMap<String, Protocol>,
}

impl TiiRegistry {
    pub fn load_dir(path: &Path) -> Result<Self, RegistryError> {
        let mut protocols = HashMap::new();

        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            let file_path = entry.path();

            if file_path.extension().and_then(|e| e.to_str()) != Some("tii") {
                continue;
            }

            let protocol =
                Protocol::from_file(&file_path).map_err(|source| RegistryError::Parse {
                    path: file_path.display().to_string(),
                    source,
                })?;

            let name = serde_json::to_value(&protocol)
                .ok()
                .and_then(|v| {
                    v.get("spec")?
                        .get("protocol")?
                        .get("name")?
                        .as_str()
                        .map(String::from)
                })
                .ok_or_else(|| RegistryError::MissingName(file_path.display().to_string()))?;

            protocols.insert(name, protocol);
        }

        Ok(Self { protocols })
    }

    pub fn get(&self, name: &str) -> Option<&Protocol> {
        self.protocols.get(name)
    }
}
