use std::fs;

use anyhow::Result;
use clap::Parser;

use picturs::diagram::parser::Diagram;
use picturs::init_logging;
use picturs::skia::A5;

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
  #[arg(short, long)]
  file: String,
}

fn main() -> Result<()> {
  init_logging();
  let args = Args::parse();
  let string = fs::read_to_string(args.file)?;
  let mut diagram = Diagram::inset(A5, (32., 32.));
  diagram.parse_string(&string);
  diagram.shrink_to_file("target/output.png");
  Ok(())
}

