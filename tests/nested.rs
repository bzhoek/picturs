#[cfg(test)]
mod tests {
  use std::fs;

  use pest::Parser;

  use picturs::{dump_rules, parse_nodes, PicParser, Rule, shapes};

  #[test]
  fn it_dumps_hierarchy() {
    let string = fs::read_to_string("tests/nested.pic").unwrap();
    let pair = PicParser::parse(Rule::picture, &*string).unwrap().next().unwrap();
    dump_rules(1, pair);
  }

  #[test]
  fn it_parses_attributes() {
    let string = fs::read_to_string("tests/homepage.pic").unwrap();
    let pair = PicParser::parse(Rule::picture, &*string).unwrap().next().unwrap();
    // dump_rules(1, pair.clone());
    let mut ast = vec![];
    let nodes = parse_nodes(pair, &mut ast);
    println!("{:?}", nodes);
  }

  #[test]
  fn it_parses_containers() {
    let string = fs::read_to_string("tests/nested.pic").unwrap();
    let pair = PicParser::parse(Rule::picture, &*string).unwrap().next().unwrap();
    let mut ast = vec![];
    let nodes = parse_nodes(pair, &mut ast);
    println!("{:?}", nodes);
  }
}