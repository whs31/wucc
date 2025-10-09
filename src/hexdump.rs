use crate::args::{HexdumpFormat, HexdumpGenerateFormat};
use anyhow::Result;
use std::io::{Read, Write};
use xxd::generate::Render;

pub fn run(args: crate::args::HexdumpArgs) -> Result<()> {
  match args.subcommand {
    crate::args::HexdumpSubcommand::Dump(a) => {
      dump(args.output_file, args.start_offset, args.length, &a)
    }
    crate::args::HexdumpSubcommand::Generate(a) => {
      generate(args.output_file, args.start_offset, args.length, &a)
    }
  }
}

fn create_reader(path: String) -> Result<Box<dyn Read>> {
  match path.as_ref() {
    "stdin" => Ok(Box::new(std::io::stdin())),
    _ => {
      let file_reader = std::fs::File::open(path)?;
      Ok(Box::new(file_reader))
    }
  }
}

fn create_writer(path: String) -> Result<Box<dyn Write>> {
  match path.as_ref() {
    "stdout" => Ok(Box::new(std::io::stdout())),
    _ => {
      let file_writer = std::fs::File::create(path)?;
      Ok(Box::new(file_writer))
    }
  }
}

pub fn dump(
  output_file: Option<String>,
  start_offset: u64,
  length: Option<u64>,
  args: &crate::args::HexdumpDumpArgs,
) -> Result<()> {
  let output_file = output_file.unwrap_or("stdout".to_string());
  let input_file = args.file.clone().unwrap_or("stdin".to_string());
  let seek = start_offset as usize;
  let settings = create_dump_settings(start_offset, args)?;
  let reader = create_reader(input_file.clone())?;
  let mut writer = create_writer(output_file.clone())?;
  match length {
    None => xxd::dump::dump_iterator(reader.bytes().skip(seek).flatten(), &mut *writer, settings),
    Some(length) => xxd::dump::dump_iterator(
      reader.bytes().skip(seek).take(length as usize).flatten(),
      &mut *writer,
      settings,
    ),
  }
}

pub fn generate(
  output_file: Option<String>,
  start_offset: u64,
  length: Option<u64>,
  args: &crate::args::HexdumpGenerateArgs,
) -> Result<()> {
  let output_file = output_file.unwrap_or("stdout".to_string());
  let input_file = args.file.clone().unwrap_or("stdin".to_string());
  let seek = start_offset as usize;
  let reader = create_reader(input_file.clone())?;
  let mut writer = create_writer(output_file.clone())?;
  let mut template = xxd::generate::Template::new(match args.template {
    HexdumpGenerateFormat::C => xxd::generate::Language::C,
    HexdumpGenerateFormat::Cpp => xxd::generate::Language::Cpp,
    HexdumpGenerateFormat::Rust => xxd::generate::Language::Rust,
    HexdumpGenerateFormat::Python => xxd::generate::Language::Python,
  });
  if let Some(prefix) = &args.prefix {
    template.set_prefix(prefix.clone());
  }
  if let Some(suffix) = &args.suffix {
    template.set_suffix(suffix.clone());
  }
  if let Some(separator) = &args.separator {
    template.set_separator(separator.clone());
  }
  if let Some(bytes_per_line) = args.line_size {
    template.set_bytes_per_line(bytes_per_line as usize);
  }
  let data: Vec<u8> = match length {
    None => reader.bytes().skip(seek).flatten().collect(),
    Some(n) => reader
      .bytes()
      .skip(seek)
      .take(n as usize)
      .flatten()
      .collect(),
  };
  writer.write_fmt(format_args!("{}\n", template.render(&data)))?;
  Ok(())
}

fn create_dump_settings(
  start_offset: u64,
  args: &crate::args::HexdumpDumpArgs,
) -> Result<xxd::dump::Config> {
  let columns = args.columns.unwrap_or(8);
  let address = start_offset;
  let group_size = args.group_size.unwrap_or(2);

  let settings = xxd::dump::Config::new()
    .format(match args.format {
      HexdumpFormat::Hex => xxd::dump::Format::Hex,
      HexdumpFormat::Bin => xxd::dump::Format::Binary,
      HexdumpFormat::Oct => xxd::dump::Format::Octal,
      HexdumpFormat::Dec => xxd::dump::Format::Decimal,
    })
    .group_size(group_size)
    .columns(columns)
    .set_address(address as usize);
  if args.plain_hexdump {
    Ok(
      settings
        .separator(false)
        .show_address(false)
        .show_interpretation(false),
    )
  } else {
    Ok(settings)
  }
}
