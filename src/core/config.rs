use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub goal: u32,
}

impl Default for Config {
    fn default() -> Self {
        Self { goal: 0 }
    }
}

fn create_config<P: AsRef<Path>>(path: P) -> Result<Config> {
    let config = Config::default();
    let s = toml::to_string(&config)?;
    fs::write(path, s.as_bytes())?;
    Ok(config)
}

pub fn read_config<P: AsRef<Path>>(path: P) -> Result<Config> {
    let path = path.as_ref();

    if path.try_exists()? {
        let read = fs::read_to_string(path)?;
        let config = toml::from_str(&read)?;
        Ok(config)
    } else {
        create_config(path)
    }
}
