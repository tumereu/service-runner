use std::net::TcpListener;
use std::path::Path;
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
            Requirement::File { paths } => {
                let workdir = self.query_service(|service| service.definition.dir.clone());

                self.perform_async_work(move || {
                   let mut errors = Vec::new();

                    for path in paths {
                        // Resolve real path: if not absolute, join with workdir
                        let pattern_path = Path::new(&path);
                        let full_pattern = if pattern_path.is_absolute() {
                            pattern_path.to_path_buf()
                        } else {
                            Path::new(&workdir).join(pattern_path)
                        };

                        // Convert to string for glob; we don't canonicalize because the user wants glob expansion.
                        let pattern_str = match full_pattern.to_str() {
                            Some(s) => s.to_owned(),
                            None => {
                                errors.push(format!("Could not convert path to string: {:?}. This indicates a bug in the runner", full_pattern));
                                continue;
                            }
                        };

                        // Convert the create path into a glob
                        match glob::glob(&pattern_str) {
                            Ok(entries) => {
                                let mut matched_any = false;
                                for entry in entries {
                                    match entry {
                                        Ok(path) => {
                                            if path.exists() {
                                                matched_any = true;
                                                break;
                                            }
                                        }
                                        Err(e) => {
                                            // e is a GlobError (e.g., I/O error while reading a directory)
                                            errors.push(format!("Unexpected IO error when checking {}: {}", pattern_str, e));
                                        }
                                    }
                                }
                                if !matched_any {
                                    errors.push(format!("No file or directory could be found using {}", pattern_str));
                                }
                            }
                            Err(e) => {
                                errors.push(format!("Error in configuration: glob pattern is invalid `{}`: {}", pattern_str, e));
                            }
                        }
                    }

                    errors.is_empty()
                }, OperationType::Check);
            }
        }
    }
}
