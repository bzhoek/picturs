use std::fs;

use clap::Parser;

use picturs::nested::Diagram;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
  #[arg(short, long)]
  file: String,
}

fn main() -> picturs::Result<()> {
  let args = Args::parse();
  let string = fs::read_to_string(args.file).unwrap();
  let mut diagram = Diagram::default();
  diagram.parse_string(&string);
  Ok(())
}

