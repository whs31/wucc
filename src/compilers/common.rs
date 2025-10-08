use std::path::PathBuf;
use anyhow::{Context, Result};

pub fn output_stem(files: &[PathBuf], output_name: &Option<String>) -> Result<String> {
  if let Some(name) = output_name {
    return Ok(name.clone());
  }

  let stems: Vec<&str> = files
    .iter()
    .map(|f| {
      f.file_stem()
        .context("Failed to get file stem")
        .and_then(|s| s.to_str().context("Failed to convert stem to string"))
    })
    .collect::<anyhow::Result<Vec<_>>>()?;

  Ok(stems.join("_"))
}