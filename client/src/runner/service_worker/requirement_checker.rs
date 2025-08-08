use reqwest::blocking::Client as HttpClient;
use reqwest::Method;
use std::net::TcpListener;
use std::path::Path;
use std::time::{Duration, Instant};
use crate::config::{ExecutableEntry, HttpMethod, Requirement};
use crate::runner::service_worker::{ConcurrentOperationStatus, WorkResult};
use crate::runner::service_worker::work_context::WorkContext;

pub enum RequirementCheckResult {
    AllOk,
    CurrentCheckOk,
    Working,
    CurrentCheckFailed,
    Timeout,
}

pub struct RequirementChecker<'a, W: WorkContext> {
    pub all_requirements: Vec<Requirement>,
    pub completed_count: usize,
    pub timeout: Option<Duration>,
    pub failure_wait_time: Duration,
    pub start_time: Instant,
    pub last_failure: Option<Instant>,
    pub context: &'a W,
    pub workdir: String,
}
impl<'a, W: WorkContext> RequirementChecker<'a, W> {
    pub fn check_requirements(self) -> RequirementCheckResult {
        let current_requirement = self.all_requirements.get(self.completed_count).cloned();
        let check_status = self.context.get_concurrent_operation_status();

        match (check_status.clone(), current_requirement, self.last_failure) {
            // If there are no more requirements to check (or there never were any at all), then we can consider the
            // check successful
            (_, None, _) => RequirementCheckResult::AllOk,
            // We have failed at least one health check, a timeout is defined, and we have exceeded it
            (_, _, Some(_)) if self.timeout.map(|timeout| {
                Instant::now().duration_since(self.start_time) > timeout
            }).unwrap_or(false) => {
                match check_status {
                    Some(ConcurrentOperationStatus::Running) => {
                        self.context.stop_concurrent_operation();
                        RequirementCheckResult::Working
                    }
                    Some(ConcurrentOperationStatus::Failed) | Some(ConcurrentOperationStatus::Ok) => {
                        self.context.clear_concurrent_operation();
                        RequirementCheckResult::Working
                    }
                    None => RequirementCheckResult::Timeout,
                }
            }
            (_, _, Some(failure_time))
                if Instant::now().duration_since(failure_time) < self.failure_wait_time =>
            {
                // We have failed at least once a short time ago. Wait for at least the specified wait time so that
                // checks are not constantly spammed.
                RequirementCheckResult::Working
            }
            // No ongoing check, still have requirements to check => start the next check
            (None, Some(requirement), _) => {
                self.check_requirement(&requirement);
                RequirementCheckResult::Working
            }
            // A check has failed, but we have not yet exceeded the timeout. Clear the operation and returned
            // corresponding status
            (Some(ConcurrentOperationStatus::Failed), _, _) => {
                self.context.clear_concurrent_operation();
                RequirementCheckResult::CurrentCheckFailed
            }
            // Current check is successful, the system can move onto the next check.
            (Some(ConcurrentOperationStatus::Ok), _, _) => {
                self.context.clear_concurrent_operation();
                RequirementCheckResult::CurrentCheckOk
            }
            (Some(ConcurrentOperationStatus::Running), _, _) => {
                // Do nothing, wait for the async check to finish
                RequirementCheckResult::Working
            }
        }
    }

    fn check_requirement(&self, requirement: &Requirement) {
        match requirement.clone() {
            Requirement::Http {
                url,
                method,
                timeout,
                status,
            } => {
                self.context.perform_concurrent_work(move || {
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
                });
            }
            Requirement::Port { port, host } => {
                let host = match host {
                    Some(host) => host.clone(),
                    None => "127.0.0.1".to_owned(),
                };

                self.context.perform_concurrent_work(
                    move || {
                        let successful = TcpListener::bind(format!("{host}:{port}")).is_err();

                        WorkResult {
                            successful,
                            output: if successful {
                                vec![format!("Req OK: successsfully bound to {host}:{port}")]
                            } else {
                                vec![format!("Req fail: could not bind to {host}:{port}")]
                            },
                        }
                    },
                );
            }
            Requirement::StateQuery { query } => {
                let result_rx = self.context.enqueue_rhai(query.clone(), true);

                self.context.perform_concurrent_work(move || {
                    match result_rx.recv() {
                        Ok(Ok(value)) if value.is::<bool>() => WorkResult {
                            successful: value.as_bool().unwrap(),
                            output: vec![format!("Query '{query}' => {value}")],
                        },
                        Ok(Ok(value)) => WorkResult {
                            successful: false,
                            output: vec![format!("Error: Query outputted non-boolean: '{query}' => {value}")],
                        },
                        Ok(Err(error)) => WorkResult {
                            successful: false,
                            output: vec![format!("Error in Rhai query {query}: {error:?}")],
                        },
                        Err(error) => WorkResult {
                            successful: false,
                            output: vec![format!("Error in receiving response from Rhai executor: {error:?}")],
                        },
                    }
                });
            }
            Requirement::File { paths } => {
                let workdir = self.workdir.clone();

                self.context.perform_concurrent_work(move || {
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
                });
            }
        };
    }
}

/// Turns a `reqwest::Error` into a clean, human-readable string.
pub fn format_reqwest_error(err: &reqwest::Error) -> String {
    if err.is_connect() {
        return format!("Connection error: {}", err);
    }
    if err.is_timeout() {
        return "Request timed out.".to_string();
    }
    if err.is_request() {
        return format!("Request build error: {}", err);
    }
    if err.is_body() {
        return format!("Body error: {}", err);
    }
    if err.is_decode() {
        return format!("Response decoding error: {}", err);
    }

    // Fall back to a generic error message
    format!("Unexpected error: {}", err)
}


pub enum SequenceEntry {
    ExecutableEntry(ExecutableEntry),
    RhaiScript(String),
    WaitRequirement {
        timeout: Duration,
        requirement: Requirement,
    },
}
