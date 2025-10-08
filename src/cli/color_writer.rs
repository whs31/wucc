use colored::Colorize;
use std::path::Path;

pub fn log_compiled_file(file: &Path, file_type: &str) {
  println!(
    "{} {} {} {}",
    "Compiled".green(),
    file_type.bold().green(),
    "file".green(),
    file
      .file_name()
      .unwrap()
      .to_string_lossy()
      .to_string()
      .bold()
  );
}
