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

    #[serde(rename = "script")]
    StateQuery {
        query: String,
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