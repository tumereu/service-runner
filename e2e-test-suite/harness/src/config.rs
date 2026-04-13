use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

use crate::terminal::TerminalSession;

/// Per-test context that manages an isolated configuration directory.
///
/// Each test gets its own subdirectory under the shared e2e mount so that
/// config files from different tests never collide.
pub struct TestContext {
    test_name: String,
    /// Host path: `/tmp/service-runner-e2e/{test_name}/config`
    config_dir: PathBuf,
    /// Container path: `/e2e/{test_name}/config`
    container_config_dir: PathBuf,
    /// Host path to the shared e2e mount root
    e2e_host_dir: PathBuf,
}

impl TestContext {
    pub fn new(test_name: &str, e2e_host_dir: &Path, e2e_container_dir: &str) -> Self {
        let config_dir = e2e_host_dir.join(test_name).join("config");
        let container_config_dir =
            PathBuf::from(e2e_container_dir).join(test_name).join("config");

        Self {
            test_name: test_name.to_string(),
            config_dir,
            container_config_dir,
            e2e_host_dir: e2e_host_dir.to_path_buf(),
        }
    }

    /// Writes a configuration file into this test's config directory.
    ///
    /// `name` may contain path separators to place files in subdirectories
    /// (e.g. `"services/my.service.yml"`). Parent directories are created
    /// automatically.
    ///
    /// Content is typically provided as an inline string literal or via
    /// [`include_str!`] referencing a fixture file next to the test source.
    pub fn write_file(&self, name: &str, content: &str) -> Result<()> {
        let path = self.config_dir.join(name);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("Creating parent dirs for {}", path.display()))?;
        }
        std::fs::write(&path, content)
            .with_context(|| format!("Writing config file {}", path.display()))?;
        Ok(())
    }

    /// Copies an existing file into this test's config directory.
    pub fn copy_file(&self, name: &str, source: &Path) -> Result<()> {
        let dest = self.config_dir.join(name);
        if let Some(parent) = dest.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::copy(source, &dest)
            .with_context(|| {
                format!(
                    "Copying {} -> {}",
                    source.display(),
                    dest.display()
                )
            })?;
        Ok(())
    }

    /// Prepares the config directory and starts the application via a fresh
    /// ttyd WebSocket session.
    ///
    /// This creates the data directory, writes the config-dir pointer that
    /// `run-app.sh` reads, and connects to ttyd — which spawns a new app
    /// process for the connection.
    pub async fn start_app(&self, ttyd_ws_url: &str) -> Result<TerminalSession> {
        self.prepare()
            .context("Preparing test context for app start")?;

        TerminalSession::connect(ttyd_ws_url, 160, 50).await
    }

    fn prepare(&self) -> Result<()> {
        // Ensure the config dir and data dir exist
        std::fs::create_dir_all(self.config_dir.join(".data"))
            .with_context(|| {
                format!(
                    "Creating .data dir under {}",
                    self.config_dir.display()
                )
            })?;

        // Write the config-dir pointer so run-app.sh knows where to look
        let pointer_path = self.e2e_host_dir.join("config_dir");
        std::fs::write(&pointer_path, self.container_config_dir.to_str().unwrap())
            .with_context(|| {
                format!(
                    "Writing config_dir pointer to {}",
                    pointer_path.display()
                )
            })?;

        log::info!(
            "[{}] Config dir ready: {} (container: {})",
            self.test_name,
            self.config_dir.display(),
            self.container_config_dir.display(),
        );

        Ok(())
    }

    /// Host path to this test's config directory (for inspecting files in
    /// assertions or post-mortem debugging).
    pub fn config_dir(&self) -> &Path {
        &self.config_dir
    }
}
