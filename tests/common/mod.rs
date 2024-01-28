use picturs::diagram::parser::{A5, Diagram};
use picturs::init_logging;

pub fn create_diagram(string: &str) -> Diagram {
  init_logging();
  let mut diagram = Diagram::inset(A5, (32., 32.));
  diagram.parse_string(string);
  diagram
}
