use std::fs;

use anyhow::Result;
use clap::Parser;

use picturs::diagram::{A5, Diagram};

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
  #[arg(short, long)]
  file: String,
}

fn main() -> Result<()> {
  let args = Args::parse();
  let string = fs::read_to_string(args.file).unwrap();
  let mut diagram = Diagram::offset(A5, (32., 32.));
  diagram.parse_string(&string);
  Ok(())
}

