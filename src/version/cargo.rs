use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

pub struct CargoFile {
  path: PathBuf,
}

impl super::VersionIO for CargoFile {
  fn new(path: &Path) -> Self {
    Self {
      path: path.to_path_buf(),
    }
  }

  fn new_auto() -> Result<Self> {
    let path_candidates = vec!["Cargo.toml"];
    let mut path = None;
    for path_candidate in path_candidates {
      let path_candidate = PathBuf::from(path_candidate);
      if path_candidate.exists() {
        path = Some(path_candidate);
        break;
      }
    }
    Ok(Self::new(
      path.context("Failed to find cargo file")?.as_path(),
    ))
  }

  fn read(&self) -> Result<semver::Version> {
    let content = std::fs::read_to_string(&self.path)
      .with_context(|| format!("Failed to read cargo file: {}", self.path.display()))?;

    // More specific regex that looks for version in [package] section
    let re = regex::Regex::new(r#"(?m)^\s*version\s*=\s*"([^"]+)"\s*$"#)
      .context("Failed to compile regex")?;

    // Find the [package] section and then look for version within it
    let mut in_package_section = false;

    for line in content.lines() {
      if line.trim().starts_with('[') {
        // Check if we're entering or leaving the package section
        in_package_section = line.trim() == "[package]";
        continue;
      }

      if in_package_section {
        if let Some(captures) = re.captures(line) {
          let version_str = captures.get(1).unwrap().as_str();
          return semver::Version::parse(version_str)
            .with_context(|| format!("Failed to parse version: {}", version_str));
        }
      }
    }

    Err(anyhow::anyhow!(
      "Version field not found in [package] section of Cargo.toml"
    ))
  }

  fn write(&self, version: &semver::Version) -> Result<()> {
    let content = std::fs::read_to_string(&self.path)
      .with_context(|| format!("Failed to read cargo file: {}", self.path.display()))?;

    // Regex to replace version specifically in the [package] section
    let re = regex::Regex::new(r#"(?m)^(\s*version\s*=\s*")[^"]+(".*)$"#)
      .context("Failed to compile regex")?;

    let mut in_package_section = false;
    let mut new_lines = Vec::new();
    let mut version_replaced = false;

    for line in content.lines() {
      let current_line = line;

      if current_line.trim().starts_with('[') {
        // Reset section tracking
        in_package_section = current_line.trim() == "[package]";
        new_lines.push(current_line.to_string());
        continue;
      }

      if in_package_section && !version_replaced {
        if let Some(captures) = re.captures(current_line) {
          let new_line = format!("{}{}{}", &captures[1], version, &captures[2]);
          new_lines.push(new_line);
          version_replaced = true;
          continue;
        }
      }

      new_lines.push(current_line.to_string());
    }

    let new_content = new_lines.join("\n");

    // Only write if the content actually changed
    if new_content != content {
      std::fs::write(&self.path, new_content)
        .with_context(|| format!("Failed to write cargo file: {}", self.path.display()))?;
    }

    Ok(())
  }
}
