use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

use regex::Regex;

use crate::config::Fingerprint;

#[derive(Debug)]
pub enum FingerprintError {
    InvalidGlobPattern(String),
    InvalidExcludePattern(String),
    IoError(PathBuf, String),
}

impl fmt::Display for FingerprintError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FingerprintError::InvalidGlobPattern(msg) => {
                write!(f, "Invalid glob pattern: {msg}")
            }
            FingerprintError::InvalidExcludePattern(msg) => {
                write!(f, "Invalid exclude regex pattern: {msg}")
            }
            FingerprintError::IoError(path, msg) => {
                write!(f, "IO error reading '{}': {msg}", path.display())
            }
        }
    }
}

pub struct FingerprintChecker {
    pub fingerprint: Fingerprint,
    pub workdir: String,
}

impl FingerprintChecker {
    pub fn new(fingerprint: Fingerprint, workdir: String) -> Self {
        Self {
            fingerprint,
            workdir,
        }
    }

    /// Calculates an MD5 checksum over the files matched by the fingerprint.
    ///
    /// Glob patterns are expanded relative to `workdir` (unless absolute). Matched
    /// paths are walked in sorted order. Any path whose workdir-relative string
    /// representation matches one of the exclude regexes is skipped entirely
    /// (directories are not descended into). Each non-excluded file contributes
    /// its relative path and raw contents to the hash.
    pub fn calculate_checksum(&self) -> Result<String, FingerprintError> {
        let exclude_patterns = self.compile_exclude_patterns()?;
        let workdir = Path::new(&self.workdir);
        let mut ctx = md5::Context::new();

        for glob_pattern in &self.fingerprint.paths {
            let full_pattern = if Path::new(glob_pattern).is_absolute() {
                glob_pattern.clone()
            } else {
                workdir.join(glob_pattern).to_string_lossy().into_owned()
            };

            let mut matches: Vec<PathBuf> = glob::glob(&full_pattern)
                .map_err(|e| FingerprintError::InvalidGlobPattern(e.to_string()))?
                .filter_map(|r| r.ok())
                .collect();
            matches.sort();

            for path in matches {
                self.hash_path(&path, workdir, &exclude_patterns, &mut ctx)?;
            }
        }

        Ok(format!("{:x}", ctx.compute()))
    }

    fn hash_path(
        &self,
        path: &Path,
        workdir: &Path,
        exclude_patterns: &[Regex],
        ctx: &mut md5::Context,
    ) -> Result<(), FingerprintError> {
        let rel = path.strip_prefix(workdir).unwrap_or(path);
        let rel_str = rel.to_string_lossy();

        if is_excluded(&rel_str, exclude_patterns) {
            return Ok(());
        }

        if path.is_file() {
            ctx.consume(rel_str.as_bytes());
            let contents = fs::read(path)
                .map_err(|e| FingerprintError::IoError(path.to_path_buf(), e.to_string()))?;
            ctx.consume(&contents);
        } else if path.is_dir() {
            let mut entries: Vec<PathBuf> = fs::read_dir(path)
                .map_err(|e| FingerprintError::IoError(path.to_path_buf(), e.to_string()))?
                .filter_map(|r| r.ok())
                .map(|e| e.path())
                .collect();
            entries.sort();

            for entry in entries {
                self.hash_path(&entry, workdir, exclude_patterns, ctx)?;
            }
        }

        Ok(())
    }

    fn compile_exclude_patterns(&self) -> Result<Vec<Regex>, FingerprintError> {
        self.fingerprint
            .exclude
            .iter()
            .map(|pat| {
                Regex::new(pat).map_err(|e| FingerprintError::InvalidExcludePattern(e.to_string()))
            })
            .collect()
    }
}

fn is_excluded(path_str: &str, exclude_patterns: &[Regex]) -> bool {
    exclude_patterns.iter().any(|re| re.is_match(path_str))
}
