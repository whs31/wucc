use anyhow::{Context, Result};
use std::{
  collections::BTreeMap,
  path::{Path, PathBuf},
};

#[allow(unused_imports)]
use askama::{Template};

#[derive(askama::Template)]
#[template(path = "resources.h", escape = "none")]
pub(crate) struct HeaderTemplate<'a> {
  pub(crate) files: &'a [String],
  pub(crate) text_getters: &'a Vec<String>,
  pub(crate) binary_getters: &'a Vec<String>,
  pub(crate) namespace: &'a str,
  pub(crate) filename: &'a str,
}

pub struct HexBytes<'a>(pub &'a Vec<u8>);

impl<'a> std::fmt::Display for HexBytes<'a> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    if self.0.is_empty() {
      return write!(f, "");
    }
    for (i, byte) in self.0.iter().enumerate() {
      if i > 0 {
        write!(f, ",\n")?;
      }
      write!(f, "0x{:02X}", byte)?;
    }
    Ok(())
  }
}

#[derive(askama::Template)]
#[template(path = "resources.cc", escape = "none")]
pub(crate) struct SourceTemplate<'a> {
  pub(crate) files: &'a [String],
  pub(crate) text_data: &'a BTreeMap<String, String>,
  pub(crate) binary_data: &'a BTreeMap<String, (usize, HexBytes<'a>)>,
  pub(crate) namespace: &'a str,
  pub(crate) filename: &'a str,
}

pub struct EmbedCompiler {
  pub namespace: String,
  pub out_dir: PathBuf
}

impl EmbedCompiler {
  pub fn new(namespace: String, out_dir: &Path) -> Self {
    Self {
      namespace,
      out_dir: out_dir.to_path_buf()
    }
  }

  fn compile_internal(
    &self,
    text_files: &[PathBuf],
    binary_files: &[PathBuf],
    stem: &str,
  ) -> Result<(String, String)> {
    let text_map = Self::read_text_files(text_files)?;
    let binary_map = Self::read_binary_files(binary_files)?;

    let mut file_strings: Vec<String> = text_files
      .iter()
      .map(|f| f.to_string_lossy().to_string())
      .collect();
    file_strings.extend(
      binary_files
        .iter()
        .map(|f| f.to_string_lossy().to_string())
    );

    let text_getters: Vec<String> = text_files
      .iter()
      .filter_map(|f| f.file_name())
      .filter_map(|s| s.to_str())
      .map(|s| s.to_string().replace(".", "_"))
      .collect();

    let binary_getters: Vec<String> = binary_files
      .iter()
      .filter_map(|f| f.file_name())
      .filter_map(|s| s.to_str())
      .map(|s| s.to_string().replace(".", "_"))
      .collect();

    let header_template = HeaderTemplate {
      files: &file_strings,
      text_getters: &text_getters,
      binary_getters: &binary_getters,
      namespace: &self.namespace,
      filename: stem,
    };

    let binary_data_wrapped: BTreeMap<String, (usize, HexBytes)> = binary_map
      .iter()
      .map(|(k, v)| (k.clone(), (v.len(), HexBytes(v))))
      .collect();

    let source_template = SourceTemplate {
      files: &file_strings,
      text_data: &text_map,
      binary_data: &binary_data_wrapped,
      namespace: &self.namespace,
      filename: stem,
    };

    let header_content = header_template.render()?;
    let source_content = source_template.render()?;

    Ok((header_content, source_content))
  }

  pub fn compile(
    &self,
    text_files: &[PathBuf],
    binary_files: &[PathBuf],
    output_name: &Option<String>,
  ) -> Result<PathBuf> {
    let mut files = text_files.to_vec();
    files.extend(binary_files.to_vec());
    let stem = super::common::output_stem(&files, output_name)?;
    let content = self.compile_internal(text_files, binary_files, &stem)?;

    let out_header_filename = format!("{}.rc.h", stem);
    let out_header_path = self.out_dir.join(out_header_filename);

    let out_source_filename = format!("{}.rc.cc", stem);
    let out_source_path = self.out_dir.join(out_source_filename);

    std::fs::create_dir_all(&self.out_dir)?;
    std::fs::write(out_header_path.clone(), content.0)?;
    std::fs::write(out_source_path.clone(), content.1)?;

    crate::cli::log_compiled_file(&out_header_path, "Resource");
    Ok(out_header_path)
  }

  fn read_text_files(files: &[PathBuf]) -> Result<BTreeMap<String, String>> {
    let mut data = BTreeMap::new();

    for file in files {
      let content = std::fs::read_to_string(file)?;

      let stem = file
        .file_name()
        .context("Failed to get file stem")?
        .to_str()
        .context("Failed to convert stem to string")?;
      data.insert(stem.to_string().replace(".", "_"), content);
    }
    Ok(data)
  }

  fn read_binary_files(files: &[PathBuf]) -> Result<BTreeMap<String, Vec<u8>>> {
    let mut data = BTreeMap::new();

    for file in files {
      let content = std::fs::read(file)?;

      let stem = file
        .file_name()
        .context("Failed to get file stem")?
        .to_str()
        .context("Failed to convert stem to string")?;
      data.insert(stem.to_string().replace(".", "_"), content);
    }
    Ok(data)
  }
}
