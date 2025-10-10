use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

pub struct ExportHeaderFile {
  path: PathBuf,
  macro_prefix: Option<String>,
}

impl super::VersionIO for ExportHeaderFile {
  fn new(path: &Path) -> Self {
    Self {
      path: path.to_path_buf(),
      macro_prefix: None,
    }
  }

  fn new_auto() -> Result<Self> {
    let search_dirs = ["./include", "./src", ".", "./inc", "./headers"];
    let target_file = "version.h";

    let path = find_file_recursive(&search_dirs, target_file).with_context(|| {
      format!(
        "Failed to find '{}' in directories: {}",
        target_file,
        search_dirs.join(", ")
      )
    })?;
    Ok(Self::new(path.as_path()))
  }

  fn read(&self) -> Result<semver::Version> {
    let content = std::fs::read_to_string(&self.path)
      .with_context(|| format!("Failed to read version header: {}", self.path.display()))?;

    let macro_prefix = self.detect_macro_prefix(&content)?;
    self
      .extract_version_from_content(&content, &macro_prefix)
      .with_context(|| format!("Failed to extract version from: {}", self.path.display()))
  }

  fn write(&self, version: &semver::Version) -> Result<()> {
    let content = std::fs::read_to_string(&self.path)
      .with_context(|| format!("Failed to read version header: {}", self.path.display()))?;

    let macro_prefix = self.detect_macro_prefix(&content)?;
    let new_content = self
      .patch_version_in_content(&content, version, &macro_prefix)
      .with_context(|| format!("Failed to patch version in: {}", self.path.display()))?;
    std::fs::write(&self.path, new_content)
      .with_context(|| format!("Failed to write version header: {}", self.path.display()))?;
    Ok(())
  }
}

impl ExportHeaderFile {
  /// Auto-detect macro prefix from file content
  fn detect_macro_prefix(&self, content: &str) -> Result<String> {
    if let Some(prefix) = &self.macro_prefix {
      return Ok(prefix.clone());
    }

    let re = regex::Regex::new(r"#\s*define\s+([A-Za-z0-9_]+)_VERSION_(MAJOR|MINOR|PATCH)\s+\d+")
      .context("Failed to create macro detection regex")?;

    let mut candidates = std::collections::HashSet::new();

    for line in content.lines() {
      if let Some(caps) = re.captures(line) {
        if let Some(prefix_match) = caps.get(1) {
          let prefix = prefix_match.as_str();
          if self.is_valid_macro_prefix(prefix) {
            candidates.insert(prefix.to_string());
          }
        }
      }
    }

    match candidates.len() {
      0 => Err(anyhow::anyhow!(
        "No version macros found in file. Expected patterns like: #define PREFIX_VERSION_MAJOR X"
      )),
      1 => Ok(candidates.into_iter().next().unwrap()),
      _ => self.select_best_prefix(candidates, content),
    }
  }

  /// Check if a string looks like a valid macro prefix
  fn is_valid_macro_prefix(&self, prefix: &str) -> bool {
    let forbidden_prefixes = ["", "_", "__", "WIN32", "LINUX", "MACOS", "DEBUG", "RELEASE"];

    !prefix.is_empty()
      && prefix.len() >= 2
      && !forbidden_prefixes.contains(&prefix)
      && prefix.chars().all(|c| c.is_ascii_uppercase() || c == '_')
  }

  /// Select the best prefix from multiple candidates
  fn select_best_prefix(
    &self,
    candidates: std::collections::HashSet<String>,
    content: &str,
  ) -> Result<String> {
    let mut prefix_counts = std::collections::HashMap::new();

    for candidate in &candidates {
      let count = content.matches(candidate).count();
      prefix_counts.insert(candidate.clone(), count);
    }

    for candidate in &candidates {
      let version_macro = format!("{}_VERSION", candidate);
      if content.contains(&version_macro) {
        *prefix_counts.get_mut(candidate).unwrap() += 10;
      }
    }

    prefix_counts
      .into_iter()
      .max_by_key(|(_, count)| *count)
      .map(|(prefix, _)| prefix)
      .context("Failed to select best macro prefix from multiple candidates")
  }

  /// Extract version from header file content using detected prefix
  fn extract_version_from_content(
    &self,
    content: &str,
    macro_prefix: &str,
  ) -> Result<semver::Version> {
    let mut major = None;
    let mut minor = None;
    let mut patch = None;

    for line in content.lines() {
      if major.is_none() {
        major = self.extract_define_value(line, macro_prefix, "MAJOR");
      }
      if minor.is_none() {
        minor = self.extract_define_value(line, macro_prefix, "MINOR");
      }
      if patch.is_none() {
        patch = self.extract_define_value(line, macro_prefix, "PATCH");
      }

      // Early exit if we found all components
      if major.is_some() && minor.is_some() && patch.is_some() {
        break;
      }
    }

    match (major, minor, patch) {
      (Some(maj), Some(min), Some(pat)) => Ok(semver::Version::new(maj, min, pat)),
      _ => Err(anyhow::anyhow!(
        "Could not find all version components for prefix '{}'. Found: major={:?}, minor={:?}, patch={:?}",
        macro_prefix,
        major,
        minor,
        patch
      )),
    }
  }

  /// Extract a specific define value from a line
  fn extract_define_value(&self, line: &str, macro_prefix: &str, suffix: &str) -> Option<u64> {
    let pattern = format!(r"#\s*define\s+{}_VERSION_{}\s+(\d+)", macro_prefix, suffix);
    let re = regex::Regex::new(&pattern).ok()?;

    re.captures(line)
      .and_then(|caps| caps.get(1))
      .and_then(|m| m.as_str().parse().ok())
  }

  /// Patch version in header file content
  fn patch_version_in_content(
    &self,
    content: &str,
    version: &semver::Version,
    macro_prefix: &str,
  ) -> Result<String> {
    let mut result = String::new();

    for line in content.lines() {
      let patched_line = self.patch_version_in_line(line, version, macro_prefix);
      result.push_str(&patched_line);
      result.push('\n');
    }

    Ok(result)
  }

  /// Patch version in a single line
  fn patch_version_in_line(
    &self,
    line: &str,
    version: &semver::Version,
    macro_prefix: &str,
  ) -> String {
    let mut result = line.to_string();

    let major_pattern = format!(r"#\s*define\s+{}_VERSION_MAJOR\s+\d+", macro_prefix);
    let major_replacement = format!("#define {}_VERSION_MAJOR {}", macro_prefix, version.major);
    if let Ok(re) = regex::Regex::new(&major_pattern) {
      result = re.replace(&result, &major_replacement).to_string();
    }

    let minor_pattern = format!(r"#\s*define\s+{}_VERSION_MINOR\s+\d+", macro_prefix);
    let minor_replacement = format!("#define {}_VERSION_MINOR {}", macro_prefix, version.minor);
    if let Ok(re) = regex::Regex::new(&minor_pattern) {
      result = re.replace(&result, &minor_replacement).to_string();
    }

    let patch_pattern = format!(r"#\s*define\s+{}_VERSION_PATCH\s+\d+", macro_prefix);
    let patch_replacement = format!("#define {}_VERSION_PATCH {}", macro_prefix, version.patch);
    if let Ok(re) = regex::Regex::new(&patch_pattern) {
      result = re.replace(&result, &patch_replacement).to_string();
    }

    result
  }
}

/// Recursively searches for a file in the given directories with error handling
fn find_file_recursive(dirs: &[&str], filename: &str) -> Result<PathBuf> {
  for dir in dirs {
    if let Some(path) = search_directory(Path::new(dir), filename)? {
      return Ok(path);
    }
  }

  Err(anyhow::anyhow!(
    "File '{}' not found in any of the specified directories",
    filename
  ))
}

/// Recursively searches a directory for a file with proper error handling
fn search_directory(dir: &Path, filename: &str) -> Result<Option<PathBuf>> {
  if !dir.exists() {
    return Ok(None);
  }

  if !dir.is_dir() {
    return Ok(None);
  }

  let entries = std::fs::read_dir(dir)
    .with_context(|| format!("Failed to read directory: {}", dir.display()))?;

  for entry in entries {
    let entry = entry.with_context(|| format!("Failed to read entry in: {}", dir.display()))?;
    let path = entry.path();

    if path.is_file() {
      if let Some(name) = path.file_name() {
        if name == filename {
          return Ok(Some(path));
        }
      }
    } else if path.is_dir() {
      let dir_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
      if !should_skip_directory(dir_name) {
        if let Some(found) = search_directory(&path, filename)? {
          return Ok(Some(found));
        }
      }
    }
  }

  Ok(None)
}

/// Skip common build and cache directories to improve performance
fn should_skip_directory(dir_name: &str) -> bool {
  matches!(
    dir_name,
    "target" | "build" | "out" | ".git" | "node_modules" | "__pycache__" | ".cache"
  )
}
