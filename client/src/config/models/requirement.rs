use std::time::Duration;
use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", deny_unknown_fields)]
pub enum Requirement {
    #[serde(rename = "http")]
    Http {
        url: String,
        method: HttpMethod,
        #[serde(with = "humantime_serde")]
        timeout: Duration,
        status: u16,
    },
    
    #[serde(rename = "port")]
    Port { port: u16, host: Option<String> },
    
    #[serde(rename = "dependency")]
    Dependency {
        #[serde(default)]
        service: Option<String>,
        block: String,
        #[serde(default)]
        status: RequiredStatus,
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub enum RequiredStatus {
    #[serde(rename = "initial")]
    Initial,
    #[serde(rename = "working")]
    Working,
    #[serde(rename = "ok")]
    Ok,
    #[serde(rename = "error")]
    Error,
}
impl Default for RequiredStatus {
    fn default() -> Self {
        Self::Ok
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum HttpMethod {
    GET,
    POST,
    PATCH,
    PUT,
    DELETE,
    OPTIONS,
}
