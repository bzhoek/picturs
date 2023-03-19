#[cfg(test)]
mod tests {
  use std::fs;

  use pest::Parser;

  use picturs::{PicParser, Rule, shapes};

  #[test]
  fn it_has_five_shapes() {
    let string = fs::read_to_string("tests/homepage.pic").unwrap();
    let pairs = PicParser::parse(Rule::picture, &*string).unwrap();
    let result = shapes(pairs);
    assert_eq!(result.len(), 5);
    let first = result.first().unwrap();
    assert_eq!(first.fit, false);
    assert_eq!(first.same, false);
    let last = result.last().unwrap();
    assert_eq!(last.fit, true);
    assert_eq!(last.same, true);
    println!("{:?}", result);
  }
}