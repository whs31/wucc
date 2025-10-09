use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

pub struct YamlMetafile {
  path: PathBuf,
}

impl super::VersionIO for YamlMetafile {
  fn new(path: &Path) -> Self {
    Self {
      path: path.to_path_buf(),
    }
  }

  fn new_auto() -> Result<Self> {
    let path_candidates = vec![
      "meta.yml",
      "meta/meta.yml",
      "plugin.yml",
      "plugin/plugin.yml",
    ];
    let mut path = None;
    for path_candidate in path_candidates {
      let path_candidate = PathBuf::from(path_candidate);
      if path_candidate.exists() {
        path = Some(path_candidate);
        break;
      }
    }
    Ok(Self::new(
      path.context("Failed to find meta file")?.as_path(),
    ))
  }

  fn read(&self) -> Result<semver::Version> {
    let content = std::fs::read_to_string(&self.path)?;
    let yaml: serde_norway::Value =
      serde_norway::from_str(&content).context("Failed to parse yaml")?;
    let version = yaml["version"]
      .as_str()
      .context("Failed to parse version")?;
    Ok(semver::Version::parse(version).context("Failed to parse version")?)
  }

  fn write(&self, version: &semver::Version) -> Result<()> {
    let content = std::fs::read_to_string(&self.path)?;
    let mut yaml: serde_norway::Value =
      serde_norway::from_str(&content).context("Failed to parse yaml")?;
    yaml["version"] = serde_norway::Value::String(version.to_string());
    let yaml = serde_norway::to_string(&yaml)?;
    std::fs::write(&self.path, yaml)?;
    Ok(())
  }
}
