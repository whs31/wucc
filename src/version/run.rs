use super::VersionIO;
use anyhow::{Context, Result};
use colored::Colorize;

fn print_version(cell_name: &str, version: &Option<semver::Version>) {
  if let Some(version) = version {
    println!(
      "• {:.<25} {}",
      cell_name,
      version.to_string().bold().magenta()
    );
  } else {
    println!("• {:.<25} {}", cell_name, "unknown".red());
  }
}

fn print_version_diff(
  cell_name: &str,
  old_version: &Option<semver::Version>,
  new_version: &Option<semver::Version>,
) {
  if let Some(old_version) = old_version {
    if let Some(new_version) = new_version {
      println!(
        "• {:.<25} {} -> {}",
        cell_name,
        old_version.to_string().bold().magenta(),
        new_version.to_string().bold().green()
      );
    } else {
      println!(
        "• {:.<25} {} -> {}",
        cell_name,
        old_version.to_string().bold().magenta(),
        "unknown".red()
      );
    }
  } else {
    if let Some(new_version) = new_version {
      println!(
        "• {:.<25} {} -> {}",
        cell_name,
        "unknown".red(),
        new_version.to_string().bold().green()
      );
    }
  }
}

pub fn min_version_present() -> Result<semver::Version> {
  let files = <dyn VersionIO>::all();
  let versions: Vec<semver::Version> = files
    .into_iter()
    .filter_map(|(_, file)| file.read().ok())
    .collect();
  versions.into_iter().min().context("No version files found")
}

pub fn run(args: crate::args::VersionArgs) -> Result<()> {
  let files = <dyn VersionIO>::all();

  if args.show {
    for (name, file) in &files {
      print_version(name, &file.read().ok());
    }
    return Ok(());
  }

  let mut ver = match args.assign {
    Some(version) => semver::Version::parse(&version)?,
    None => min_version_present().unwrap_or_else(|_| semver::Version::new(0, 1, 0)),
  };
  
  if args.bump_major {
    ver = semver::Version::new(ver.major + 1, 0, 0);
  } else if args.bump_minor {
    ver = semver::Version::new(ver.major, ver.minor + 1, 0);
  } else if args.bump_patch {
    ver = semver::Version::new(ver.major, ver.minor, ver.patch + 1);
  }

  for (name, file) in files {
    let old_version = file.read().ok();
    print_version_diff(name, &old_version, &Some(ver.clone()));
    let _ = file.write(&ver);
  }

  Ok(())
}
