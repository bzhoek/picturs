use std::fs;

use anyhow::Result;
use clap::Parser;

use picturs::diagram::Diagram;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
  #[arg(short, long)]
  file: String,
}

fn main() -> Result<()> {
  let args = Args::parse();
  let string = fs::read_to_string(args.file).unwrap();
  let mut diagram = Diagram::default();
  diagram.parse_string(&string);
  Ok(())
}

