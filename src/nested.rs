use std::error::Error;

use pest::iterators::{Pair, Pairs};
use pest_derive::Parser;

type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[derive(Parser)]
#[grammar = "nested.pest"] // relative to project `src`
pub struct NestedParser;

#[derive(PartialEq)]
#[derive(Debug)]
pub enum Node<'a> {
  Primitive(Option<&'a str>, Shape<'a>),
  Container(Option<&'a str>, Vec<Node<'a>>),
}

#[derive(PartialEq)]
#[derive(Debug)]
pub enum Shape<'a> {
  Line(Option<&'a str>, &'a str, &'a str),
  Rectangle(Option<&'a str>),
}

pub fn traverse_nested<'a>(pairs: Pairs<'a, Rule>, mut ast: Vec<Node<'a>>) -> Vec<Node<'a>> {
  for pair in pairs.into_iter() {
    match pair.as_rule() {
      Rule::container => {
        let title = find_rule(&pair, Rule::inner);
        let nodes = traverse_nested(pair.into_inner(), vec![]);
        ast.push(Node::Container(title, nodes));
      }
      Rule::line => {
        let id = find_rule(&pair, Rule::id);
        let source = find_rule(&pair, Rule::source).unwrap();
        let target = find_rule(&pair, Rule::target).unwrap();
        ast.push(Node::Primitive(id,
          Shape::Line(id, source, target))
        );
      }
      Rule::rectangle => {
        let title = find_rule(&pair, Rule::inner);
        let id = find_rule(&pair, Rule::id);
        ast.push(Node::Primitive(id, Shape::Rectangle(title)));
      }
      _ => {
        println!("unmatched {:?}", pair);
        ast = traverse_nested(pair.into_inner(), ast);
      }
    }
  }
  ast
}

fn find_rule<'a>(pair: &Pair<'a, Rule>, rule: Rule) -> Option<&'a str> {
  let title = pair.clone().into_inner()
    .find(|p| p.as_rule() == rule)
    .map(|p| p.as_str());
  title
}

fn dump_nested(level: usize, pairs: Pairs<Rule>) {
  for pair in pairs.into_iter() {
    println!("{:level$} {:?}", level, pair);
    dump_nested(level + 1, pair.into_inner());
  }
}

#[cfg(test)]
mod tests {
  use pest::Parser;

  use crate::nested::{NestedParser, Node, Rule, Shape, traverse_nested};
  use crate::nested::Node::{Container, Primitive};

  use super::*;

  fn parse_nested(string: &str) -> Result<Vec<Node>> {
    let top = nested_top(string)?;
    let ast = traverse_nested(top.clone(), vec![]);
    Ok(ast)
  }

  fn nested_top(string: &str) -> Result<Pairs<Rule>> {
    Ok(NestedParser::parse(Rule::picture, &*string)?)
  }

  #[test]
  fn single_box_untitled() -> Result<()> {
    let string = r#"box"#;
    let ast = parse_nested(string)?;
    assert_eq!(ast, vec![
      Primitive(None, Shape::Rectangle(None)),
    ]);
    Ok(())
  }

  #[test]
  fn single_box_id() -> Result<()> {
    let string = r#"box.first "title""#;
    let top = nested_top(string)?;
    dump_nested(0, top);

    let ast = parse_nested(string)?;
    assert_eq!(ast, vec![
      Primitive(Some("first"), Shape::Rectangle(Some("title"))),
    ]);
    Ok(())
  }

  #[test]
  fn single_box_with_title() -> Result<()> {
    let string = r#"box "title""#;
    let ast = parse_nested(string)?;
    assert_eq!(ast, vec![
      Primitive(None, Shape::Rectangle(Some("title"))),
    ]);
    Ok(())
  }

  #[test]
  fn double_box() -> Result<()> {
    let string = "box
                         box";
    let ast = parse_nested(string)?;
    assert_eq!(ast, vec![
      Primitive(None, Shape::Rectangle(None)),
      Primitive(None, Shape::Rectangle(None)),
    ]);
    Ok(())
  }

  #[test]
  fn nested_box_untitled() -> Result<()> {
    let string = "box { box }";
    let ast = parse_nested(string)?;
    assert_eq!(ast, vec![
      Container(None, vec![Primitive(None, Shape::Rectangle(None))])
    ]);
    Ok(())
  }

  #[test]
  fn nested_box_id() -> Result<()> {
    let string = "box.parent { box }";
    let ast = parse_nested(string)?;
    assert_eq!(ast, vec![
      Container(None, vec![Primitive(None, Shape::Rectangle(None))])
    ]);
    Ok(())
  }

  #[test]
  fn nested_box_with_title() -> Result<()> {
    let string = r#"box "parent" { box "child" }"#;
    let ast = parse_nested(string)?;
    assert_eq!(ast, vec![
      Container(Some("parent"), vec![Primitive(None, Shape::Rectangle(Some("child")))])
    ]);
    Ok(())
  }

  #[test]
  fn line_from_a_to_b() -> Result<()> {
    let string = r#"line from now.n to future.n"#;
    let ast = parse_nested(string)?;
    assert_eq!(ast, vec![
      Primitive(None, Shape::Line(None, "now.n", "future.n")),
    ]);
    Ok(())
  }

  #[test]
  fn dump_nested_box_with_title() -> Result<()> {
    let string = r#"box "parent" { box "child" }"#;
    let top = nested_top(string)?;
    dump_nested(0, top);
    Ok(())
  }

  #[test]
  fn parse_extended_example() -> Result<()> {
    let string =
      r#"box.now "Now" {
        box.step3 "What do we need to start doing now"
      }
      box.future "March" {
        box.step1 "Imagine it is four months into the future"
        box.step2 "What would you like to write about the past period"
        box.note "IMPORTANT: write in past tense"
      }
      line from now.n to future.n
      "#;
    nested_top(string)?;
    Ok(())
  }
}