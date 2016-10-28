use error;
use serde_json;
use std::path::Path;
use std::fs::File;
use std::collections::BTreeMap;

#[derive(Debug)]
pub struct Configuration {
    api_key: String,
}

impl Configuration {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Configuration, error::Error> {
        let file = try!(File::open(path));
        let mut config: BTreeMap<String, String> = try!(serde_json::from_reader(file));
        match config.remove("api_key") {
            Some(api_key) => {
                Ok(Configuration{api_key: api_key })
            },
            None => Err(error::Error::from("api_key missing from configuration file".to_owned()))
        }
    }

    pub fn api_key(&self) ->  &String {
        &self.api_key
    }
}
