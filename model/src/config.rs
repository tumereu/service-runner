use std::collections::HashMap;
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