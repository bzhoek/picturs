use std::fs;

use clap::Parser;

use picturs::nested::parse_nested;
use picturs::skia::Canvas;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
  #[arg(short, long)]
  file: String,
}

fn main() -> picturs::Result<()> {
  let args = Args::parse();
  let string = fs::read_to_string(args.file).unwrap();
  let mut canvas = Canvas::new(400, 800);
  parse_nested(&string, &mut canvas)?;
  Ok(())
}

