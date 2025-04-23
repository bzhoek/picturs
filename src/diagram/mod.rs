use crate::diagram::parser::Diagram;
use crate::init_logging;
use crate::skia::A5;

pub mod types;
pub mod parser;
pub mod rules;
pub mod conversion;
pub mod renderer;
pub mod index;
pub mod edges;
mod attributes;
pub mod bounds;

pub fn create_diagram(string: &str) -> Diagram {
  init_logging();
  let mut diagram = Diagram::inset(A5, (16., 16.));
  diagram.parse_string(string);
  diagram
}
