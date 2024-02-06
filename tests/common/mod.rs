use picturs::diagram::parser::Diagram;
use picturs::init_logging;
use picturs::skia::A5;

pub fn create_diagram(string: &str) -> Diagram {
  init_logging();
  let mut diagram = Diagram::inset(A5, (16., 16.));
  diagram.parse_string(string);
  diagram
}
