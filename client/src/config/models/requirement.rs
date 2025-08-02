use std::fmt;
use std::fmt::{Display, Formatter};
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

    #[serde(rename = "file")]
    File {
        paths: Vec<String>
    },

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
impl Display for RequiredStatus {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Initial => f.write_str("Initial"),
            Self::Working => f.write_str("Working"),
            Self::Ok => f.write_str("Ok"),
            Self::Error => f.write_str("Error"),
        }
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
impl Display for HttpMethod {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let s = match self {
            HttpMethod::GET => "GET",
            HttpMethod::POST => "POST",
            HttpMethod::PATCH => "PATCH",
            HttpMethod::PUT => "PUT",
            HttpMethod::DELETE => "DELETE",
            HttpMethod::OPTIONS => "OPTIONS",
        };
        write!(f, "{}", s)
    }
}