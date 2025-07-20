use crate::config::{AutomationEntry, ExecutableEntry, HealthCheck, HealthCheckConfig, HttpMethod};
use serde_derive::{Deserialize, Serialize};
use crate::config::models::dependency::{Dependency, RequiredStatus};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct ServiceDefinition {
    pub name: String,
    pub dir: String,
    pub stages: Vec<Stage>,
    #[serde(default = "Vec::new")]
    pub automation: Vec<AutomationEntry>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Stage {
    pub name: String,
    pub health: Option<HealthCheckConfig>,
    #[serde(default)]
    pub prerequisites: Vec<Dependency>,
    #[serde(flatten)]
    pub work: StageWork,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", deny_unknown_fields)]
pub enum StageWork {
    #[serde(rename = "cmd-seq")]
    CommandSeq { commands: Vec<ExecutableEntry> },
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
          - name: "run"
            type: "process"
            prerequisites:
              - stage: build
            executable:
              executable: "./target/debug/myservice"
              args: []
            health:
              timeout_millis: 60000
              checks:
                - type: "port"
                  port: 8080
                - type: "http"
                  url: "http://localhost:8080/health"
                  method: GET
                  timeout_millis: 1000
                  status: 200
    "#;

    let result: ServiceDefinition = serde_yaml::from_str(yaml).expect("Failed to deserialize");

    assert_eq!(result.name, "MyService");
    assert_eq!(result.dir, "./services/myservice");
    assert_eq!(result.stages.len(), 2);

    let build_stage = &result.stages[0];
    match &build_stage.work {
        StageWork::CommandSeq { commands } => {
            assert_eq!(commands.len(), 2);
            assert_eq!(commands[0].executable, "cargo");
            assert_eq!(commands[0].args, vec!["clean"]);
        }
        _ => panic!("Expected CmdSeq variant for stage 0"),
    }

    let run_stage = &result.stages[1];
    match &run_stage.work {
        StageWork::Process { executable } => {
            assert_eq!(executable.executable, "./target/debug/myservice");
        }
        _ => panic!("Expected Process variant for stage 1"),
    }

    assert!(build_stage.health.is_none(), "Build stage should have no health check");

    let run_checks = run_stage
        .health
        .as_ref()
        .expect("Expected health checks on run stage");
    assert_eq!(run_checks.timeout_millis, 60000);
    assert_eq!(run_checks.checks.len(), 2);
    assert!(matches!(
        run_checks.checks[0],
        HealthCheck::Port { port: 8080 }
    ));

    match &run_checks.checks[1] {
        HealthCheck::Http { url, method, status, .. } => {
            assert_eq!(url, "http://localhost:8080/health");
            assert_eq!(*status, 200);
            assert!(matches!(method, HttpMethod::GET));
        }
        _ => panic!("Expected HTTP health check"),
    }

    // Check the prereq arrays for both stages
    assert_eq!(build_stage.prerequisites.len(), 0);
    assert_eq!(run_stage.prerequisites.len(), 1);
    assert_eq!(run_stage.prerequisites[0].status, RequiredStatus::Ok);
    assert_eq!(run_stage.prerequisites[0].service, None);
    assert_eq!(run_stage.prerequisites[0].stage, "build");
}
