#[cfg(test)]
mod tests {
  use std::fs;

  use pest::Parser;

  use picturs::{dump_rules, parse_nodes, PicParser, Rule};
  use picturs::Node::{Attribute, Primitive, String};
  use picturs::Shape::{Arrow, Box};

  #[test]
  fn it_dumps_homepage() {
    let string = fs::read_to_string("tests/homepage.pic").unwrap();
    let pair = PicParser::parse(Rule::picture, &*string).unwrap().next().unwrap();
    dump_rules(1, pair);
  }

  #[test]
  fn it_parses_attributes() {
    let string = fs::read_to_string("tests/homepage.pic").unwrap();
    let pair = PicParser::parse(Rule::picture, &*string).unwrap().next().unwrap();
    let nodes = parse_nodes(pair, vec![]);

    assert_eq!(nodes, vec![
      Primitive(Arrow,
        vec![Attribute("right 200%".to_string()), String("Markdown".to_string()), String("Source".to_string())]),
      Primitive(Box,
        vec![String("Markdown".to_string()), String("Formatter".to_string()), String("(markdown.c)".to_string())]),
      Primitive(Arrow,
        vec![Attribute("right 200%".to_string()), String("HTML+SVG".to_string()), String("Output".to_string())]),
      Primitive(Arrow,
        vec![Attribute("down 70%".to_string()), Attribute("from last box.s".to_string())]),
      Primitive(Box,
        vec![String("Pikchr".to_string()), String("Formatter".to_string()), String("(pikchr.c)".to_string())]),
    ])
  }
}