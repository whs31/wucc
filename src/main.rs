use anyhow::Result;

mod args;
pub(crate) mod cli;
mod compilers;

fn main() -> Result<()> {
  let args = args::parse_args();

  match args.subcommand {
    args::Subcommand::JsonToCpp(args) => {
      let c =
        compilers::json::JsonCompiler::new(args.namespace.clone(), &args.output_dir, args.nlohmann);

      c.compile(args.input.as_slice(), &args.output_name)?;
    }
    args::Subcommand::YamlToCpp(args) => {
      let c =
        compilers::yaml::YamlCompiler::new(args.namespace.clone(), &args.output_dir, args.nlohmann);

      c.compile(args.input.as_slice(), &args.output_name)?;
    }
  }

  Ok(())
}
