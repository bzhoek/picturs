use std::fs;

use clap::Parser;

use picturs::nested::parse_nested;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
  #[arg(short, long)]
  file: String,
}

fn main() -> picturs::Result<()> {
  let args = Args::parse();
  let string = fs::read_to_string(args.file).unwrap();
  parse_nested(&string)?;
  Ok(())
}

