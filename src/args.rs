use clap::Parser;
use std::path::PathBuf;

pub fn parse_args() -> Args {
  Args::parse()
}

#[derive(clap::Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
#[command(next_line_help = false)]
#[command(color = clap::ColorChoice::Auto)]
pub struct Args {
  #[command(subcommand)]
  pub subcommand: Subcommand,
}

#[derive(clap::Subcommand, Debug, Clone)]
pub enum Subcommand {
  JsonToCpp(JsonToCppCompileArgs),
  YamlToCpp(YamlToCppCompileArgs),
  Embed(EmbedCompileArgs),
  Hexdump(HexdumpArgs),
  Version(VersionArgs),

  #[clap(hide = true)]
  WhoIsTheBest,
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
  #[arg(short = 'O', long)]
  pub output_dir: PathBuf,

  /// Output file name.
  #[arg(short = 'o', long)]
  pub output_name: Option<String>,

  /// Generate nlohmann::json object as well.
  #[arg(long)]
  pub nlohmann: bool,
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
  #[arg(short = 'O', long)]
  pub output_dir: PathBuf,

  /// Output file name.
  #[arg(short = 'o', long)]
  pub output_name: Option<String>,

  /// Generate nlohmann::json object as well.
  #[arg(long)]
  pub nlohmann: bool,
}

#[derive(clap::Args, Debug, Clone)]
pub struct EmbedCompileArgs {
  /// Path to the input text file(s).
  #[arg(short, long, num_args = 1..)]
  pub text: Vec<PathBuf>,

  /// Path to the input binary file(s).
  #[arg(short, long, num_args = 1..)]
  pub binary: Vec<PathBuf>,

  /// Namespace to use.
  #[arg(short, long)]
  pub namespace: String,

  /// Path to the output directory.
  #[arg(short = 'O', long)]
  pub output_dir: PathBuf,

  /// Output file name.
  #[arg(short = 'o', long)]
  pub output_name: Option<String>,
}

#[derive(clap::Args, Debug, Clone)]
pub struct HexdumpArgs {
  /// File to which output will be written (default: stdout).
  #[arg(short = 'o', long = "output-file")]
  pub output_file: Option<String>,

  /// Offset to start at (default: 0).
  #[arg(short = 's', long = "seek", default_value_t = 0)]
  pub start_offset: u64,

  /// Amount of bytes to read (default: all).
  #[arg(short = 'l', long = "length")]
  pub length: Option<u64>,

  /// Subcommand to use.
  #[command(subcommand)]
  pub subcommand: HexdumpSubcommand,
}

#[derive(clap::Subcommand, Debug, Clone)]
pub enum HexdumpSubcommand {
  Dump(HexdumpDumpArgs),
  Generate(HexdumpGenerateArgs),
}

#[derive(clap::Args, Debug, Clone)]
pub struct HexdumpDumpArgs {
  /// File to read from (default: stdin).
  #[arg(value_name = "FILE")]
  pub file: Option<String>,

  /// Output in postscript hexdump format.
  #[arg(short = 'p', long = "plain-hexdump")]
  pub plain_hexdump: bool,

  /// Specifies the output format for the value (default: hex).
  #[arg(
        short = 'f',
        long = "format",
        value_enum,
        default_value_t = HexdumpFormat::Hex
  )]
  pub format: HexdumpFormat,

  /// Separate the output of every <bytes> bytes (two hex characters or eight bit-digits each) by a whitespace.
  #[arg(short = 'g', long = "group-size", value_name = "BYTES")]
  pub group_size: Option<usize>,

  /// Specifies the amount of output columns.
  #[arg(short = 'c', long = "columns", value_name = "COUNT")]
  pub columns: Option<usize>,
}

#[derive(clap::Args, Debug, Clone)]
pub struct HexdumpGenerateArgs {
  /// File to read from (default: stdin).
  #[arg(value_name = "FILE")]
  pub file: Option<String>,

  /// Specifies a template file which shall be used for a generation (default: C).
  #[arg(
        short = 't',
        long = "template",
        value_enum,
        default_value_t = HexdumpGenerateFormat::C
  )]
  pub template: HexdumpGenerateFormat,

  /// Specifies a custom prefix for the template.
  #[arg(long = "set-prefix")]
  pub prefix: Option<String>,

  /// Specifies a custom suffix for the template.
  #[arg(long = "set-suffix")]
  pub suffix: Option<String>,

  /// Specifies a custom separator for the template.
  #[arg(long = "set-separator")]
  pub separator: Option<String>,

  /// Specifies a custom number of bytes per line for the template.
  #[arg(long = "set-bytes")]
  pub line_size: Option<u64>,
}

#[derive(clap::ValueEnum, Clone, Debug)]
pub enum HexdumpFormat {
  Hex,
  Bin,
  Oct,
  Dec,
}

impl std::str::FromStr for HexdumpFormat {
  type Err = String;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s.to_lowercase().as_str() {
      "hex" => Ok(HexdumpFormat::Hex),
      "bin" => Ok(HexdumpFormat::Bin),
      "oct" => Ok(HexdumpFormat::Oct),
      "dec" => Ok(HexdumpFormat::Dec),
      _ => Err(format!("Invalid format: {}", s)),
    }
  }
}

#[derive(clap::ValueEnum, Clone, Debug)]
pub enum HexdumpGenerateFormat {
  C,
  Cpp,
  Rust,
  Python,
}

impl std::str::FromStr for HexdumpGenerateFormat {
  type Err = String;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s.to_lowercase().as_str() {
      "c" => Ok(HexdumpGenerateFormat::C),
      "cpp" => Ok(HexdumpGenerateFormat::Cpp),
      "rust" => Ok(HexdumpGenerateFormat::Rust),
      "python" => Ok(HexdumpGenerateFormat::Python),
      _ => Err(format!("Invalid format: {}", s)),
    }
  }
}

#[derive(clap::Args, Debug, Clone)]
pub struct VersionArgs {
  /// Show versions and exit.
  #[arg(short = 's', long = "show", conflicts_with_all = ["assign", "bump_patch", "bump_minor", "bump_major"])]
  pub show: bool,

  /// Assign new version.
  #[arg(short = 'a', long = "assign", conflicts_with_all = ["show", "bump_patch", "bump_minor", "bump_major"])]
  pub assign: Option<String>,

  /// Bump version patch.
  #[arg(short = 'p', long = "bump-patch", conflicts_with_all = ["show", "assign", "bump_minor", "bump_major"])]
  pub bump_patch: bool,

  /// Bump version minor.
  #[arg(short = 'm', long = "bump-minor", conflicts_with_all = ["show", "assign", "bump_patch", "bump_major"])]
  pub bump_minor: bool,

  /// Bump version major.
  #[arg(short = 'M', long = "bump-major", conflicts_with_all = ["show", "assign", "bump_patch", "bump_minor"])]
  pub bump_major: bool,
}
