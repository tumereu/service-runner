use std::net::TcpListener;
use std::path::Path;
use std::sync::{Arc, Mutex};
use reqwest::blocking::Client as HttpClient;
use reqwest::Method;

use crate::config::{HttpMethod, Requirement};
use crate::models::BlockStatus;
use crate::rhai::RHAI_ENGINE;
use crate::runner::service_worker::block_worker::BlockWorker;
use crate::runner::service_worker::utils::format_reqwest_error;
use crate::runner::service_worker::WorkResult;
use crate::system_state::OperationType;

pub enum RequirementCheckResult {
    Ok,
    Failed,
    Async,
}

pub trait RequirementChecker {
    fn check_requirement(&self, requirement: &Requirement, silent: bool);
}

impl RequirementChecker for BlockWorker {
    fn check_requirement(
        &self,
        requirement: &Requirement,
        silent: bool,
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
                            &url,
                        )
                        .timeout(timeout.clone())
                        .send();

                    match result {
                        Ok(response) if response.status().as_u16() == status => {
                            WorkResult {
                                successful: true,
                                output: vec![
                                    format!("Req OK: {method} {url} responded with status {status}")
                                ]
                            }
                        }
                        Ok(response) => {
                            let response_status: u16 = response.status().as_u16();

                            WorkResult {
                                successful: false,
                                output: vec![
                                    format!("Req fail: {method} {url} responded with status {response_status} != {status}")
                                ]
                            }
                        }
                        Err(error) => {
                            WorkResult {
                                successful: false,
                                output: vec![format_reqwest_error(&error)]
                            }
                        }
                    }
                }, OperationType::Check, silent);
            }
            Requirement::Port { port, host } => {
                let host = match host {
                    Some(host) => host.clone(),
                    None => "127.0.0.1".to_owned()
                };

                // TODO output to logs
                self.perform_async_work(move || {
                    let successful = TcpListener::bind(format!("{host}:{port}")).is_err();

                    WorkResult {
                        successful,
                        output: if successful {
                            vec![format!("Req OK: successsfully bound to {host}:{port}")]
                        } else {
                            vec![format!("Req fail: could not bind to {host}:{port}")]
                        }
                    }
                }, OperationType::Check, silent);
            }
            Requirement::StateQuery { query } => {
                let mut scope = self.create_rhai_scope();
                // TODO currently evaluation is performed synchronously. Move engine to a worker thread to allow for
                // longer scripts?
                let result = match RHAI_ENGINE.eval_with_scope::<bool>(&mut scope, &query) {
                    Ok(value) => WorkResult {
                        successful: value,
                        output: vec![
                            format!("Query evaluated to {value}")
                        ]
                    },
                    Err(e) => WorkResult {
                        successful: false,
                        output: vec![
                            format!("Error processing expression: {e:?}")
                        ]
                    }
                };

                self.perform_async_work(move || result, OperationType::Check, silent);
            }
            Requirement::File { paths } => {
                let workdir = self.query_service(|service| service.definition.dir.clone());

                self.perform_async_work(move || {
                    let mut output = Vec::new();
                    let mut success = true;

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
                                output.push(format!("Could not convert path to string: {:?}. This indicates a bug in the runner", full_pattern));
                                success = false;
                                continue;
                            }
                        };

                        // Convert the create path into a glob
                        success = match glob::glob(&pattern_str) {
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
                                            output.push(format!("Req fail: unexpected IO error when checking {}: {}", pattern_str, e));
                                        }
                                    }
                                }
                                if matched_any {
                                    output.push(format!("Req OK: path {} exists", pattern_str));
                                } else {
                                    output.push(format!("Req fail: no file/dir found with '{}'", pattern_str));
                                }

                                matched_any
                            }
                            Err(_) => {
                                output.push(format!("Req fail: invalid glob pattern '{}'", pattern_str));
                                false
                            }
                        } && success
                    }

                    WorkResult {
                        successful: success,
                        output
                    }
                }, OperationType::Check, silent);
            }
        };
    }
}
