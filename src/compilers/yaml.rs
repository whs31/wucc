use anyhow::{Context, Result};
use std::{
  collections::BTreeMap,
  path::{Path, PathBuf},
};
use askama::Template;

pub struct YamlCompiler {
  pub namespace: String,
  pub out_dir: PathBuf,
  pub with_nlohmann: bool,
}

impl YamlCompiler {
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
    let jsons = Self::read_yaml_files(files)?;

    let file_strings: Vec<String> = files
      .iter()
      .map(|f| f.to_string_lossy().to_string())
      .collect();

    let template = super::json::HeaderTemplate {
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

    let out_filename = format!("{}.yml.h", stem);
    let out_path = self.out_dir.join(out_filename);

    std::fs::create_dir_all(&self.out_dir)?;
    std::fs::write(out_path.clone(), content)?;

    crate::cli::log_compiled_file(&out_path, "YAML");
    Ok(out_path)
  }

  fn read_yaml_files(files: &[PathBuf]) -> Result<BTreeMap<String, String>> {
    let mut json_data = BTreeMap::new();

    for file in files {
      let content = std::fs::read_to_string(file)?;
      let yaml: serde_norway::Value = serde_norway::from_str(&content)?;
      let json = serde_json::to_string(&yaml)?;

      let stem = file
        .file_stem()
        .context("Failed to get file stem")?
        .to_str()
        .context("Failed to convert stem to string")?;
      json_data.insert(stem.to_string(), json);
    }
    Ok(json_data)
  }
}
