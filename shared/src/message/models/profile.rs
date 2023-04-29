use std::convert::Into;
use std::fmt::Write;

use serde::{Deserialize, Serialize};

use crate::config::{Profile as ConfigProfile, Service as ConfigService};
use crate::message::models::Service;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Profile {
    pub name: String,
    pub services: Vec<Service>,
}
impl Profile {
    pub fn new(profile: &ConfigProfile, all_services: &Vec<ConfigService>) -> Profile {
        let services: Vec<Service> = all_services
            .iter()
            .filter(|service| profile.includes(service))
            .map(|service| service.clone().into())
            .collect();

        Profile {
            name: profile.name.clone(),
            services,
        }
    }
}
