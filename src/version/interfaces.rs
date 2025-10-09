use anyhow::Result;
use std::path::Path;

pub trait VersionIO {
  fn new(path: &Path) -> Self
  where
    Self: Sized;
  fn new_auto() -> Result<Self>
  where
    Self: Sized;
  fn read(&self) -> Result<semver::Version>;
  fn write(&self, version: &semver::Version) -> Result<()>;
}

impl dyn VersionIO {
  pub fn all() -> Vec<(&'static str, Box<dyn VersionIO>)> {
    let mut files = Vec::new();
    if let Ok(meta) = super::YamlMetafile::new_auto() {
      files.push(("Plugin metafile", Box::new(meta) as Box<dyn VersionIO>));
    }
    if let Ok(cargo) = super::CargoFile::new_auto() {
      files.push(("Cargo manifest", Box::new(cargo) as Box<dyn VersionIO>));
    }
    if let Ok(cmake) = super::CmakeFile::new_auto() {
      files.push(("CMakeLists", Box::new(cmake) as Box<dyn VersionIO>));
    }
    if let Ok(conan) = super::ConanFile::new_auto() {
      files.push(("Conanfile", Box::new(conan) as Box<dyn VersionIO>));
    }

    files
  }
}
