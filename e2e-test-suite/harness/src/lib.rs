pub mod config;
pub mod docker;
pub mod terminal;

use anyhow::{Context, Result};
use std::path::PathBuf;
use std::time::Duration;
use tokio::sync::OnceCell;

use docker::DockerManager;

const CONTAINER_NAME: &str = "service-runner-e2e";
const IMAGE_NAME: &str = "service-runner-e2e";
const TTYD_PORT: u16 = 7681;
const E2E_HOST_DIR: &str = "/tmp/service-runner-e2e";
const E2E_CONTAINER_DIR: &str = "/e2e";

/// Top-level test environment. Lazily initialized once per process via [`TestEnv::get`].
///
/// Handles Docker image build, container lifecycle, and provides factory
/// methods for per-test contexts.
pub struct TestEnv {
    #[allow(dead_code)]
    docker: DockerManager,
    e2e_host_dir: PathBuf,
    ttyd_port: u16,
}

static INSTANCE: OnceCell<TestEnv> = OnceCell::const_new();

impl TestEnv {
    /// Returns a shared reference to the singleton test environment.
    ///
    /// On first call this builds the Docker image, starts the container, and
    /// waits for ttyd to become reachable. Subsequent calls return immediately.
    pub async fn get() -> &'static TestEnv {
        INSTANCE
            .get_or_init(|| async {
                env_logger::try_init().ok();
                TestEnv::init()
                    .await
                    .expect("Failed to initialize e2e test environment")
            })
            .await
    }

    async fn init() -> Result<Self> {
        let e2e_host_dir = PathBuf::from(E2E_HOST_DIR);
        std::fs::create_dir_all(&e2e_host_dir)
            .context("Failed to create e2e host directory")?;

        let docker = DockerManager::new(IMAGE_NAME, CONTAINER_NAME)?;

        log::info!("Building Docker image '{IMAGE_NAME}'...");
        docker.build_image()?;

        log::info!("Ensuring container '{CONTAINER_NAME}' is running...");
        docker.ensure_container_running(&e2e_host_dir, TTYD_PORT)?;

        log::info!("Waiting for ttyd to be ready on port {TTYD_PORT}...");
        Self::wait_for_ttyd(TTYD_PORT, Duration::from_secs(30)).await?;

        log::info!("Test environment ready.");
        Ok(Self {
            docker,
            e2e_host_dir,
            ttyd_port: TTYD_PORT,
        })
    }

    async fn wait_for_ttyd(port: u16, timeout: Duration) -> Result<()> {
        let url = format!("http://localhost:{port}");
        let start = tokio::time::Instant::now();
        loop {
            if reqwest::get(&url).await.is_ok() {
                return Ok(());
            }
            if start.elapsed() > timeout {
                anyhow::bail!("Timeout waiting for ttyd on port {port}");
            }
            tokio::time::sleep(Duration::from_millis(200)).await;
        }
    }

    /// Creates a new per-test context with an isolated config directory.
    pub fn new_test_context(&self, test_name: &str) -> config::TestContext {
        config::TestContext::new(test_name, &self.e2e_host_dir, E2E_CONTAINER_DIR)
    }

    /// WebSocket URL for connecting to the ttyd instance in the container.
    pub fn ttyd_ws_url(&self) -> String {
        format!("ws://localhost:{}/ws", self.ttyd_port)
    }
}
