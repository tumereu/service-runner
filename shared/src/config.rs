use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::{BufReader, Result as IOResult};
use std::path::Path;
use serde::Deserialize;
use Vec;

#[derive(Deserialize, Debug)]
pub struct ServiceConfig {
    pub services: HashMap<String, Service>
}


#[derive(Deserialize, Debug)]
pub struct Service {
    #[serde(default)]
    pub depends_on: Vec<String>,
    pub compile: Option<String>,
    pub run: String
}

pub fn read_config<P: AsRef<Path>>(path: P) -> Result<ServiceConfig, Box<dyn Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    Ok(serde_yaml::from_reader(reader)?)
}
