use error;
use rustc_serialize::json;
use std::path::Path;
use std::fs::File;
use std::io::Read;

#[derive(RustcDecodable, RustcEncodable, Debug)]
pub struct Configuration {
    api_key: String,
}

impl Configuration {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Configuration, error::Error> {
        let mut file = try!(File::open(path));
        let mut data = String::new();
        try!(file.read_to_string(&mut data));
        Ok(try!(json::decode(&data)))
    }

    pub fn api_key(&self) ->  &String {
        &self.api_key
    }
}
