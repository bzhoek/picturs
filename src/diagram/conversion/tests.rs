use crate::diagram::conversion::Conversion;
use crate::diagram::parser::Rule;

mod fractions {
  use crate::diagram::conversion::Conversion;
  use crate::diagram::parser::Rule;
  use crate::diagram::types::{Edge, ObjectEdge};

  #[test]
  fn with_horizontal_fraction() {
    let object = subject("from 1/4 a.s").unwrap();
    let mut edge = Edge::below();
    edge.x = -0.25;
    assert_eq!(object, ObjectEdge::new("a", edge));
  }

  #[test]
  fn with_vertical_fraction() {
    let object = subject("from 1/4 a.w").unwrap();
    let mut edge = Edge::left();
    edge.y = -0.25;
    assert_eq!(object, ObjectEdge::new("a", edge));
  }

  fn subject(string: &str) -> Option<ObjectEdge> {
    let pair = Conversion::pair_for(Rule::line_attributes, string);
    Conversion::location_to_edge(&pair, Rule::source)
  }
}

mod degrees {
  use super::*;

  #[test]
  fn with_edge() {
    let edge = subject("a.20");
    assert_eq!(edge, ("a", Some(20)));

    let edge = subject("b.nw");
    assert_eq!(edge, ("b", Some(315)));

    let edge = subject("c.3:");
    assert_eq!(edge, ("c", Some(90)))
  }

  #[test]
  fn without_edge() {
    let edge = subject("d");
    assert_eq!(edge, ("d", None))
  }

  fn subject(string: &str) -> (&str, Option<i32>) {
    let pair = Conversion::pair_for(Rule::object_edge, string);
    Conversion::object_edge_in_degrees_from(pair)
  }
}
