use std::convert::Into;

use serde::{Deserialize, Serialize};

use crate::config::{ProfileDefinition, ServiceDefinition};
use crate::models::Service;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Profile {
    pub definition: ProfileDefinition,
    pub services: Vec<Service>,
}
impl Profile {
    pub fn new(profile: ProfileDefinition, all_services: &Vec<ServiceDefinition>) -> Profile {
        let services: Vec<Service> = profile.services
            .iter()
            .flat_map(|service_ref| {
                all_services.iter()
                    .find(|service| &service.name == &service_ref.name)
                    .map(|service| service.to_owned().into())
                    .into_iter()
            })
            .collect();

        Profile {
            definition: profile,
            services,
        }
    }
}
