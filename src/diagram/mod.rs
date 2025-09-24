use crate::diagram::parser::Diagram;
use crate::diagram::types::{Length, Unit};
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

pub fn create_diagram(string: &str) -> Diagram<'_> {
  init_logging();
  let pad = Length::new(1., Unit::Pc).pixels();
  let mut diagram = Diagram::inset(A5, (pad, pad));
  diagram.parse_string(string);
  diagram
}
