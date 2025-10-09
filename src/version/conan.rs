use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

pub struct ConanFile {
  path: PathBuf,
}

impl super::VersionIO for ConanFile {
  fn new(path: &Path) -> Self {
    Self {
      path: path.to_path_buf(),
    }
  }

  fn new_auto() -> Result<Self> {
    let path_candidates = vec!["conanfile.py"];
    let mut path = None;
    for path_candidate in path_candidates {
      let path_candidate = PathBuf::from(path_candidate);
      if path_candidate.exists() {
        path = Some(path_candidate);
        break;
      }
    }
    Ok(Self::new(
      path.context("Failed to find conan file")?.as_path(),
    ))
  }

  fn read(&self) -> Result<semver::Version> {
    let content = std::fs::read_to_string(&self.path)
      .with_context(|| format!("Failed to read conanfile: {}", self.path.display()))?;

    let re = regex::Regex::new(r#"(?m)^\s*version\s*=\s*"([^"]+)"\s*$"#)
      .context("Failed to compile regex")?;
    let mut in_conan_class = false;

    for line in content.lines() {
      if line.trim().starts_with("class") && line.contains("ConanFile") {
        in_conan_class = true;
        continue;
      }

      if line.trim().starts_with("class") && in_conan_class {
        in_conan_class = false;
        if line.contains("ConanFile") {
          in_conan_class = true;
        }
        continue;
      }

      if in_conan_class {
        if let Some(captures) = re.captures(line) {
          let version_str = captures.get(1).unwrap().as_str();
          return semver::Version::parse(version_str)
            .with_context(|| format!("Failed to parse version: {}", version_str));
        }
      }
    }

    Err(anyhow::anyhow!(
      "Version field not found in ConanFile class"
    ))
  }

  fn write(&self, version: &semver::Version) -> Result<()> {
    let content = std::fs::read_to_string(&self.path)
      .with_context(|| format!("Failed to read conanfile: {}", self.path.display()))?;

    let re = regex::Regex::new(r#"(?m)^(\s*version\s*=\s*")[^"]+(".*)$"#)
      .context("Failed to compile regex")?;

    let mut in_conan_class = false;
    let mut new_lines = Vec::new();
    let mut version_replaced = false;

    for line in content.lines() {
      let current_line = line;

      if current_line.trim().starts_with("class") && current_line.contains("ConanFile") {
        in_conan_class = true;
        new_lines.push(current_line.to_string());
        continue;
      }

      if current_line.trim().starts_with("class") && in_conan_class {
        in_conan_class = false;
        if current_line.contains("ConanFile") {
          in_conan_class = true;
        }
        new_lines.push(current_line.to_string());
        continue;
      }

      if in_conan_class && !version_replaced {
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
    if new_content != content {
      std::fs::write(&self.path, new_content)
        .with_context(|| format!("Failed to write conanfile: {}", self.path.display()))?;
    }

    Ok(())
  }
}
