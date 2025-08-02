use std::net::TcpListener;
use std::time::Duration;
use crate::config::{HttpMethod, RequiredStatus, Requirement};
use crate::runner::service_worker::block_worker::BlockWorker;
use reqwest::blocking::Client as HttpClient;
use reqwest::Method;
use crate::models::BlockStatus;

pub enum RequirementCheckResult {
    Ok,
    Failed,
    Async,
}

pub trait RequirementChecker {
    fn check_requirement(&self, requirement: &Requirement) -> bool;
}

impl RequirementChecker for BlockWorker {
    fn check_requirement(
        &self,
        requirement: &Requirement,
    ) -> bool {
        match requirement {
            Requirement::Http {
                url,
                method,
                timeout_millis,
                status,
            } => {
                let http_client = HttpClient::new();

                let result = http_client
                    .request(
                        match method {
                            HttpMethod::GET => Method::GET,
                            HttpMethod::POST => Method::POST,
                            HttpMethod::PUT => Method::PUT,
                            HttpMethod::PATCH => Method::PATCH,
                            HttpMethod::DELETE => Method::DELETE,
                            HttpMethod::OPTIONS => Method::OPTIONS,
                        },
                        url,
                    )
                    .timeout(Duration::from_millis(*timeout_millis))
                    .send();

                // TODO include http call in logs, also block and service id?
                if let Ok(response) = result {
                    let response_status: u16 = response.status().into();
                    if response_status != *status {
                        self.add_ctrl_output(
                            format!(
                                "Requirement failed: HTTP status {actual} != {expected}",
                                actual = response_status,
                                expected = status
                            ),
                        );

                        false
                    } else {
                        self.add_ctrl_output(
                            format!(
                                "Requirement OK: HTTP status {actual} == {expected}",
                                actual = response_status,
                                expected = status
                            ),
                        );

                        true
                    }
                } else {
                    self.add_ctrl_output(
                        "Requirement failed: HTTP request timeout".to_string(),
                    );

                    false
                }
            }
            Requirement::Port { port, host } => {
                let host = match host {
                    Some(host) => host.clone(),
                    None => "127.0.0.1".to_owned()
                };

                if TcpListener::bind(format!("{host}:{port}")).is_err() {
                    self.add_ctrl_output(format!("Requirement failed: port {host}:{port} not open"));

                    false
                } else {
                    self.add_ctrl_output(format!("Requirement OK: port {host}:{port} is open"));

                    true
                }
            }
            Requirement::Dependency { service: required_service, block: block_ref, status: required_status } => {
                let result = self.query_system(|system| {
                    system.iter_services()
                        // Find the service the prerequisite refers to
                        .find(|service| match required_service {
                            // Default to the service itself
                            None => service.definition.id == self.service_id,
                            Some(req_service_id) => &service.definition.id == req_service_id
                        })
                        .map(|service| {
                            // Check that the status is acceptable according to the required status of the prereq
                            match service.get_block_status(block_ref) {
                                BlockStatus::Initial => *required_status == RequiredStatus::Initial,
                                BlockStatus::Working { .. } => *required_status == RequiredStatus::Working,
                                BlockStatus::Ok => *required_status == RequiredStatus::Ok,
                                BlockStatus::Error => *required_status == RequiredStatus::Error,
                            }
                        })
                        .unwrap_or(false);
                });

                result
            }
        }
    }
}
