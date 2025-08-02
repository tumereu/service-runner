use crate::config::{AutomationEntry, ExecutableEntry, Requirement, HttpMethod};
use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct ServiceDefinition {
    pub id: String,
    pub dir: String,
    pub blocks: Vec<Block>,
    #[serde(default = "Vec::new")]
    pub automation: Vec<AutomationEntry>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Block {
    pub id: String,
    pub status_line: StatusLineDefinition,
    #[serde(default)]
    pub health: HealthCheckConfig,
    #[serde(default)]
    pub prerequisites: Vec<Requirement>,
    #[serde(flatten)]
    pub work: WorkDefinition,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(deny_unknown_fields)]
pub struct HealthCheckConfig {
    #[serde(default)]
    pub timeout_millis: u64,
    pub requirements: Vec<Requirement>
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StatusLineDefinition {
    pub symbol: String,
    pub slot: usize,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", deny_unknown_fields)]
pub enum WorkDefinition {
    #[serde(rename = "cmd-seq")]
    CommandSeq { commands: Vec<ExecutableEntry> },
    #[serde(rename = "process")]
    Process { executable: ExecutableEntry },
}

#[test]
fn test_deserialize_service_definition() {
    let yaml = r#"
        id: "test-service"
        dir: "./services/test-service"
        automation: []
        blocks:
          - id: "build"
            type: "cmd-seq"
            status_line:
              symbol: "B"
              slot: 1
            commands:
              - executable: "cargo"
                args: ["clean"]
                env:
                  RUST_BACKTRACE: "1"
              - executable: "cargo"
                args: ["build"]
          - id: "run"
            type: "process"
            status_line:
              symbol: "R"
              slot: 0
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

    assert_eq!(result.id, "test-service");
    assert_eq!(result.dir, "./services/test-service");
    assert_eq!(result.blocks.len(), 2);

    let build_stage = &result.blocks[0];
    match &build_stage.work {
        WorkDefinition::CommandSeq { commands } => {
            assert_eq!(commands.len(), 2);
            assert_eq!(commands[0].executable, "cargo");
            assert_eq!(commands[0].args, vec!["clean"]);
        }
        _ => panic!("Expected CmdSeq variant for stage 0"),
    }

    let run_stage = &result.blocks[1];
    match &run_stage.work {
        WorkDefinition::Process { executable } => {
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
    assert_eq!(run_checks.requirements.len(), 2);
    assert!(matches!(
        run_checks.requirements[0],
        Requirement::Port { port: 8080 }
    ));

    match &run_checks.requirements[1] {
        Requirement::Http { url, method, status, .. } => {
            assert_eq!(url, "http://localhost:8080/health");
            assert_eq!(*status, 200);
            assert!(matches!(method, HttpMethod::GET));
        }
        _ => panic!("Expected HTTP health check"),
    }

    // Check the prereq arrays for both stages
    assert_eq!(build_stage.prerequisites.len(), 0);
    assert_eq!(run_stage.prerequisites.len(), 1);
    // FIXME test new fields, modify example yaml

    // Check the status line entries for both stages
    assert_eq!(build_stage.status_line.symbol, "B");
    assert_eq!(build_stage.status_line.slot, 1);

    assert_eq!(run_stage.status_line.symbol, "R");
    assert_eq!(run_stage.status_line.slot, 0);
}
