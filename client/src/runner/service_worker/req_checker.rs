use std::net::TcpListener;
use std::time::Duration;
use crate::config::{HttpMethod, RequiredStatus, Requirement};
use crate::runner::service_worker::block_worker::BlockWorker;
use reqwest::blocking::Client as HttpClient;
use reqwest::Method;
use crate::models::BlockStatus;
use crate::system_state::OperationType;

pub enum RequirementCheckResult {
    Ok,
    Failed,
    Async,
}

pub trait RequirementChecker {
    fn check_requirement(&self, requirement: &Requirement);
}

impl RequirementChecker for BlockWorker {
    fn check_requirement(
        &self,
        requirement: &Requirement,
    ) {
        match requirement.clone() {
            Requirement::Http {
                url,
                method,
                timeout,
                status,
            } => {
                self.perform_async_work(move || {
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
                        .timeout(timeout.clone())
                        .send();

                    // TODO output to logs
                    if let Ok(response) = result {
                        let response_status: u16 = response.status().into();
                        response_status != status
                    } else {
                        false
                    }
                }, OperationType::Check);
            }
            Requirement::Port { port, host } => {
                let host = match host {
                    Some(host) => host.clone(),
                    None => "127.0.0.1".to_owned()
                };

                // TODO output to logs
                self.perform_async_work(move || {
                   TcpListener::bind(format!("{host}:{port}")).is_err()
                }, OperationType::Check);
            }
            Requirement::Dependency { service: required_service, block: block_ref, status: required_status } => {
                let result = self.query_system(|system| {
                    system.iter_services()
                        // Find the service the prerequisite refers to
                        .find(|service| match &required_service {
                            // Default to the service itself
                            None => service.definition.id == self.service_id,
                            Some(req_service_id) => &service.definition.id == req_service_id
                        })
                        .map(|service| {
                            // Check that the status is acceptable according to the required status of the prereq
                            match service.get_block_status(&block_ref) {
                                BlockStatus::Initial => required_status == RequiredStatus::Initial,
                                BlockStatus::Working { .. } => required_status == RequiredStatus::Working,
                                BlockStatus::Ok => required_status == RequiredStatus::Ok,
                                BlockStatus::Error => required_status == RequiredStatus::Error,
                            }
                        })
                        .unwrap_or(false)
                });

                self.perform_async_work(move || {
                    result
                }, OperationType::Check);
            }
        }
    }
}
