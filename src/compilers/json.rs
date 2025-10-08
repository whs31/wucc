use anyhow::{Context, Result};
use std::{
  collections::BTreeMap,
  path::{Path, PathBuf},
};

#[allow(unused_imports)]
use askama::{filters, Template};

#[derive(askama::Template)]
#[template(path = "json_header.h", escape = "none")]
pub(crate) struct HeaderTemplate<'a> {
  pub(crate) files: &'a [String],
  pub(crate) json_data: &'a BTreeMap<String, String>,
  pub(crate) namespace: &'a str,
  pub(crate) output_stem: &'a str,
  pub(crate) with_nlohmann: bool,
}

pub struct JsonCompiler {
  pub namespace: String,
  pub out_dir: PathBuf,
  pub with_nlohmann: bool,
}

impl JsonCompiler {
  pub fn new(namespace: String, out_dir: &Path, with_nlohmann: bool) -> Self {
    Self {
      namespace,
      out_dir: out_dir.to_path_buf(),
      with_nlohmann,
    }
  }

  fn compile_internal(
    &self,
    files: &[PathBuf],
    stem: &str,
  ) -> Result<String> {
    let jsons = Self::read_json_files(files)?;

    let file_strings: Vec<String> = files
      .iter()
      .map(|f| f.to_string_lossy().to_string())
      .collect();

    let template = HeaderTemplate {
      files: &file_strings,
      json_data: &jsons,
      namespace: &self.namespace,
      output_stem: stem,
      with_nlohmann: self.with_nlohmann,
    };
    Ok(template.render()?)
  }

  pub fn compile(
    &self,
    files: &[PathBuf],
    output_name: &Option<String>,
  ) -> Result<PathBuf> {
    let stem = super::common::output_stem(files, output_name)?;
    let content = self.compile_internal(files, &stem)?;

    let out_filename = format!("{}.json.h", stem);
    let out_path = self.out_dir.join(out_filename);

    std::fs::create_dir_all(&self.out_dir)?;
    std::fs::write(out_path.clone(), content)?;

    crate::cli::log_compiled_file(&out_path, "JSON");
    Ok(out_path)
  }

  fn output_stem(files: &[PathBuf], output_name: &Option<String>) -> Result<String> {
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
      .collect::<Result<Vec<_>>>()?;

    Ok(stems.join("_"))
  }

  fn read_json_files(files: &[PathBuf]) -> Result<BTreeMap<String, String>> {
    let mut json_data = BTreeMap::new();

    for file in files {
      let content = std::fs::read_to_string(file)?;
      let _parsed: serde_json::Value = serde_json::from_str(&content)?;

      let stem = file
        .file_stem()
        .context("Failed to get file stem")?
        .to_str()
        .context("Failed to convert stem to string")?;
      json_data.insert(stem.to_string(), content);
    }
    Ok(json_data)
  }
}
