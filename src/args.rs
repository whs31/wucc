use std::path::PathBuf;
use clap::Parser;

pub fn parse_args() -> Args {
  Args::parse()
}

#[derive(clap::Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
#[command(next_line_help = false)]
#[command(color = clap::ColorChoice::Auto)]
pub struct Args {
  #[command(subcommand)]
  pub subcommand: Subcommand
}

#[derive(clap::Subcommand, Debug, Clone)]
pub enum Subcommand {
  JsonToCpp(JsonToCppCompileArgs),
  YamlToCpp(YamlToCppCompileArgs),
}

#[derive(clap::Args, Debug, Clone)]
pub struct JsonToCppCompileArgs {
  /// Path to the input file.
  #[arg(value_name = "INPUT", num_args = 1..)]
  pub input: Vec<PathBuf>,

  /// Namespace to use.
  #[arg(short, long)]
  pub namespace: String,

  /// Path to the output directory.
  #[arg(short='O', long)]
  pub output_dir: PathBuf,

  /// Output file name.
  #[arg(short='o', long)]
  pub output_name: Option<String>,

  /// Generate nlohmann::json object as well.
  #[arg(long)]
  pub nlohmann: bool
}

#[derive(clap::Args, Debug, Clone)]
pub struct YamlToCppCompileArgs {
  /// Path to the input file.
  #[arg(value_name = "INPUT", num_args = 1..)]
  pub input: Vec<PathBuf>,

  /// Namespace to use.
  #[arg(short, long)]
  pub namespace: String,

  /// Path to the output directory.
  #[arg(short='O', long)]
  pub output_dir: PathBuf,

  /// Output file name.
  #[arg(short='o', long)]
  pub output_name: Option<String>,

  /// Generate nlohmann::json object as well.
  #[arg(long)]
  pub nlohmann: bool
}