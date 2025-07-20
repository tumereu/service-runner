use crate::config::{AutomationEntry, ExecutableEntry, HealthCheck, HealthCheckConfig, HttpMethod};
use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct ServiceDefinition {
    pub name: String,
    pub dir: String,
    pub stages: Vec<StageDefinition>,
    #[serde(default = "Vec::new")]
    pub automation: Vec<AutomationEntry>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StageDefinition {
    pub name: String,
    pub checks: Option<HealthCheckConfig>,
    #[serde(flatten)]
    pub work: StageWorkDefinition,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", deny_unknown_fields)]
pub enum StageWorkDefinition {
    #[serde(rename = "cmd-seq")]
    CmdSeq { commands: Vec<ExecutableEntry> },
    #[serde(rename = "process")]
    Process { executable: ExecutableEntry },
}

#[test]
fn test_deserialize_service_definition() {
    let yaml = r#"
        name: "MyService"
        dir: "./services/myservice"
        automation: []
        stages:
          - name: "build"
            type: "cmd-seq"
            commands:
              - executable: "cargo"
                args: ["clean"]
                env:
                  RUST_BACKTRACE: "1"
              - executable: "cargo"
                args: ["build"]
                env: {}
            checks:
              timeout_millis: 5000
              checks:
                - type: "http"
                  url: "http://localhost:8080/health"
                  method: GET
                  timeout_millis: 1000
                  status: 200
          - name: "run"
            type: "process"
            executable:
              executable: "./target/debug/myservice"
              args: []
              env: {}
            checks:
              timeout_millis: 2000
              checks:
                - type: "port"
                  port: 8080
    "#;

    let result: ServiceDefinition = serde_yaml::from_str(yaml).expect("Failed to deserialize");

    // Basic structure assertions
    assert_eq!(result.name, "MyService");
    assert_eq!(result.dir, "./services/myservice");
    assert_eq!(result.stages.len(), 2);

    let build_stage = &result.stages[0];
    match &build_stage.work {
        StageWorkDefinition::CmdSeq { commands } => {
            assert_eq!(commands.len(), 2);
            assert_eq!(commands[0].executable, "cargo");
            assert_eq!(commands[0].args, vec!["clean"]);
        }
        _ => panic!("Expected CmdSeq variant for stage 0"),
    }

    let run_stage = &result.stages[1];
    match &run_stage.work {
        StageWorkDefinition::Process { executable } => {
            assert_eq!(executable.executable, "./target/debug/myservice");
        }
        _ => panic!("Expected Process variant for stage 1"),
    }

    // Check health checks
    let build_checks = build_stage
        .checks
        .as_ref()
        .expect("Expected health checks on build stage");
    assert_eq!(build_checks.timeout_millis, 5000);
    assert_eq!(build_checks.checks.len(), 1);

    match &build_checks.checks[0] {
        HealthCheck::Http {
            url,
            method,
            status,
            ..
        } => {
            assert_eq!(url, "http://localhost:8080/health");
            assert_eq!(*status, 200);
            assert!(matches!(method, HttpMethod::GET));
        }
        _ => panic!("Expected HTTP health check"),
    }

    let run_checks = run_stage
        .checks
        .as_ref()
        .expect("Expected health checks on run stage");
    assert_eq!(run_checks.timeout_millis, 2000);
    assert_eq!(run_checks.checks.len(), 1);
    assert!(matches!(
        run_checks.checks[0],
        HealthCheck::Port { port: 8080 }
    ));
}
