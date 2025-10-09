use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

pub struct CmakeFile {
  path: PathBuf,
}

impl super::VersionIO for CmakeFile {
  fn new(path: &Path) -> Self {
    Self {
      path: path.to_path_buf(),
    }
  }

  fn new_auto() -> Result<Self> {
    let path_candidates = vec!["CMakeLists.txt"];
    let mut path = None;
    for path_candidate in path_candidates {
      let path_candidate = PathBuf::from(path_candidate);
      if path_candidate.exists() {
        path = Some(path_candidate);
        break;
      }
    }
    Ok(Self::new(
      path.context("Failed to find cmake file")?.as_path(),
    ))
  }

  fn read(&self) -> Result<semver::Version> {
    let content = std::fs::read_to_string(&self.path)
      .with_context(|| format!("Failed to read cmake file: {}", self.path.display()))?;

    let re = regex::Regex::new(r#"(?i)\bproject\s*\(\s*\w+\s+(?:.*\s+)?VERSION\s+(\d+\.\d+\.\d+)"#)
      .context("Failed to compile regex")?;

    if let Some(captures) = re.captures(&content) {
      let version_str = captures.get(1).unwrap().as_str();
      semver::Version::parse(version_str)
        .with_context(|| format!("Failed to parse version: {}", version_str))
    } else {
      Err(anyhow::anyhow!(
        "VERSION field not found in project() section of CMakeLists.txt"
      ))
    }
  }

  fn write(&self, version: &semver::Version) -> Result<()> {
    let content = std::fs::read_to_string(&self.path)
      .with_context(|| format!("Failed to read CMakeLists.txt: {}", self.path.display()))?;

    let re =
      regex::Regex::new(r#"(?i)(\bproject\s*\(\s*\w+\s+(?:.*\s+)?VERSION\s+)(\d+\.\d+\.\d+)"#)
        .context("Failed to compile regex")?;

    let new_content = re.replace(&content, format!("${{1}}{}", version));

    if new_content != content {
      std::fs::write(&self.path, new_content.as_bytes())
        .with_context(|| format!("Failed to write CMakeLists.txt: {}", self.path.display()))?;
    }

    Ok(())
  }
}
