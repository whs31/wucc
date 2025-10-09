use anyhow::Result;

mod args;
pub(crate) mod cli;
mod compilers;
mod hexdump;

fn main() -> Result<()> {
  human_panic::setup_panic!();
  let args = args::parse_args();

  match args.subcommand {
    args::Subcommand::JsonToCpp(a) => {
      let c =
        compilers::json::JsonCompiler::new(a.namespace.clone(), &a.output_dir, a.nlohmann);

      c.compile(a.input.as_slice(), &a.output_name)?;
    }
    args::Subcommand::YamlToCpp(a) => {
      let c =
        compilers::yaml::YamlCompiler::new(a.namespace.clone(), &a.output_dir, a.nlohmann);

      c.compile(a.input.as_slice(), &a.output_name)?;
    }
    args::Subcommand::Hexdump(a) => hexdump::run(a)?
  }

  Ok(())
}
