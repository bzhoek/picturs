use picturs::diagram::conversion::Conversion;
use picturs::diagram::parser::Rule;

#[cfg(test)]
mod rules {
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