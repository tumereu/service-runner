use std::collections::{HashMap, VecDeque};
use std::convert::Into;
use std::fmt::{Display, Formatter, Write};

use serde::{Deserialize, Serialize};
use toml::value::Index;

use crate::config::{
    HttpMethod as ConfigHttpMethod,
    ExecutableEntry as ConfigExecutableEntry, Profile as ConfigProfile, Service as ConfigService, ScriptedRunConfig as ConfigScriptedRunConfig, HealthCheck as ConfigHealthCheck, Dependency as ConfigDependency, RequiredState as ConfigRequiredState, ScriptedCompileConfig as ConfigScriptedCompileConfig};
use crate::message::models::Service;
use crate::write_escaped_str;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Profile {
    pub name: String,
    pub services: Vec<Service>
}
impl Profile {
    pub fn new(profile: &ConfigProfile, all_services: &Vec<ConfigService>) -> Profile {
        let services: Vec<Service> = all_services.iter()
            .filter(|service| profile.includes(service))
            .map(|service| service.clone().into())
            .collect();

        Profile {
            name: profile.name.clone(),
            services
        }
    }
}
