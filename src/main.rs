use std::fs;
use std::path::PathBuf;
use anyhow::Result;
use clap::Parser;
use log::info;
use picturs::diagram::parser::Diagram;
use picturs::init_logging;
use picturs::skia::A5;

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
  input: Option<PathBuf>,
  #[arg(short, long)]
  output: Option<PathBuf>,
}

fn main() -> Result<()> {
  init_logging();
  let args = Args::parse();
  if let Some(input) = args.input {
    let string = fs::read_to_string(&input)?;
    let mut diagram = Diagram::inset(A5, (32., 32.));
    diagram.parse_string(&string);
    let output = args.output.unwrap_or_else(|| input.with_extension("png"));
    diagram.shrink_to_file(output.as_os_str().to_str().unwrap());
    info!("Wrote diagram to {:?}", output);
  };
  Ok(())
}

