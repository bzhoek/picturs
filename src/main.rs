use std::{fs, io};
use std::path::PathBuf;
use std::io::{Read};

use anyhow::Result;
use clap::Parser;
use log::info;
use picturs::diagram::parser::Diagram;
use picturs::init_logging;
use picturs::skia::A5;

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
  #[arg(short, long)]
  input: Option<PathBuf>,
  #[arg(short, long)]
  output: Option<PathBuf>,
}

fn main() -> Result<()> {
  init_logging();
  let args = Args::parse();
  let mut string = String::new();
  if let Some(path) = args.input {
    string = fs::read_to_string(&path)?;
  } else {
    io::stdin().read_to_string(&mut string)?;
  };
  let mut diagram = Diagram::inset(A5, (32., 32.));
  diagram.parse_string(&string);

  let output = args.output.expect("Output path is required");
  diagram.shrink_to_file(output.as_os_str().to_str().unwrap(), None);
  info!("Wrote diagram to {:?}", output);
  Ok(())
}

