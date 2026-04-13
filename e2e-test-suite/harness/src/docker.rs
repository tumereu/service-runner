use anyhow::{bail, Context, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

pub struct DockerManager {
    image_name: String,
    container_name: String,
    repo_root: PathBuf,
    dockerfile_path: PathBuf,
}

impl DockerManager {
    pub fn new(image_name: &str, container_name: &str) -> Result<Self> {
        let repo_root = Self::find_repo_root()?;
        let dockerfile_path = repo_root.join("e2e-test-suite/Dockerfile");

        if !dockerfile_path.exists() {
            bail!(
                "Dockerfile not found at {}",
                dockerfile_path.display()
            );
        }

        Ok(Self {
            image_name: image_name.to_string(),
            container_name: container_name.to_string(),
            repo_root,
            dockerfile_path,
        })
    }

    fn find_repo_root() -> Result<PathBuf> {
        let output = Command::new("git")
            .args(["rev-parse", "--show-toplevel"])
            .output()
            .context("Failed to run git")?;

        if !output.status.success() {
            bail!("Not in a git repository");
        }

        let path = String::from_utf8(output.stdout)
            .context("Invalid UTF-8 in git output")?
            .trim()
            .to_string();

        Ok(PathBuf::from(path))
    }

    pub fn build_image(&self) -> Result<()> {
        let status = Command::new("docker")
            .args([
                "build",
                "-t",
                &self.image_name,
                "-f",
                self.dockerfile_path.to_str().unwrap(),
                ".",
            ])
            .current_dir(&self.repo_root)
            .status()
            .context("Failed to run docker build. Is Docker installed and running?")?;

        if !status.success() {
            bail!("Docker image build failed");
        }

        Ok(())
    }

    /// Ensures a container is running with the current image.
    ///
    /// If a container already exists with the same image, it is reused.
    /// Otherwise the old container is removed and a fresh one is started.
    pub fn ensure_container_running(&self, e2e_host_dir: &Path, ttyd_port: u16) -> Result<()> {
        if self.is_container_running_with_current_image()? {
            log::info!("Container already running with current image — reusing");
            return Ok(());
        }

        self.stop_and_remove_container()?;

        let mount = format!("{}:{}", e2e_host_dir.display(), "/e2e");
        let port_map = format!("{ttyd_port}:7681");

        let output = Command::new("docker")
            .args([
                "run",
                "-d",
                "--name",
                &self.container_name,
                "-p",
                &port_map,
                "-v",
                &mount,
                &self.image_name,
            ])
            .output()
            .context("Failed to start container")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            bail!("Failed to start container: {stderr}");
        }

        Ok(())
    }

    fn is_container_running_with_current_image(&self) -> Result<bool> {
        let container_image = match Command::new("docker")
            .args([
                "inspect",
                "--format",
                "{{.Image}}",
                &self.container_name,
            ])
            .output()
        {
            Ok(o) if o.status.success() => {
                String::from_utf8_lossy(&o.stdout).trim().to_string()
            }
            _ => return Ok(false),
        };

        let state = match Command::new("docker")
            .args([
                "inspect",
                "--format",
                "{{.State.Running}}",
                &self.container_name,
            ])
            .output()
        {
            Ok(o) if o.status.success() => {
                String::from_utf8_lossy(&o.stdout).trim().to_string()
            }
            _ => return Ok(false),
        };

        if state != "true" {
            return Ok(false);
        }

        let current_image = match Command::new("docker")
            .args(["inspect", "--format", "{{.Id}}", &self.image_name])
            .output()
        {
            Ok(o) if o.status.success() => {
                String::from_utf8_lossy(&o.stdout).trim().to_string()
            }
            _ => return Ok(false),
        };

        Ok(container_image == current_image)
    }

    fn stop_and_remove_container(&self) -> Result<()> {
        let exists = Command::new("docker")
            .args(["inspect", &self.container_name])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if exists {
            log::info!("Stopping and removing previous container '{}'", self.container_name);
            let _ = Command::new("docker")
                .args(["stop", &self.container_name])
                .output();
            let _ = Command::new("docker")
                .args(["rm", "-f", &self.container_name])
                .output();
        }

        Ok(())
    }
}
