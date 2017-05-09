use error;
use serde_json;
use std::path::Path;
use std::fs::File;

#[derive(Debug, Deserialize)]
pub struct Configuration {
    pub api_key: String,
    pub interval: u64,
}

impl Configuration {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Configuration, error::Error> {
        let file = File::open(path)?;
        let config: Configuration = serde_json::from_reader(file)?;
        Ok(config)
    }
}
