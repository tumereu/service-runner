use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct Dependency {
    #[serde(default)]
    pub service: Option<String>,
    pub stage: String,
    #[serde(default)]
    pub status: RequiredStatus,
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

