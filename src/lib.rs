use std::error::Error;

use pest::iterators::{Pair, Pairs};
use pest_derive::Parser;

#[allow(dead_code)]
type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[derive(Parser)]
#[grammar = "pic.pest"] // relative to project `src`
pub struct PicParser;

#[derive(Debug)]
pub enum Attribute {
  Same
}

#[derive(Debug)]
pub enum Node {
  Primitive(ShapeType),
  Container(Vec<Node>),
}

#[derive(Debug)]
pub enum ShapeType {
  Arc,
  Arrow,
  Box,
  Unset,
}

#[derive(Debug)]
pub struct Shape {
  class: ShapeType,
  text: Vec<String>,
  path: Vec<String>,
  pub fit: bool,
  pub same: bool, // same styling for this /**/`class`
}

impl Default for Shape {
  fn default() -> Self {
    Shape {
      class: ShapeType::Unset,
      text: Vec::new(),
      path: Vec::new(),
      fit: false,
      same: false,
    }
  }
}

pub fn parse_nodes<'a>(pair: Pair<'a, Rule>, ast: &'a mut Vec<Node>) -> &'a mut Vec<Node> {
  let mut pairs = pair.into_inner();
  while pairs.peek().is_some() {
    let pair = pairs.next().unwrap();
    match pair.as_rule() {
      Rule::container => {
        let attr = pairs.next().unwrap();
        println!("container {:?}", pair);
        println!("attr {:?}", attr);
        let mut children: Vec<Node> = vec![];
        parse_nodes(pair, &mut children);
        ast.push(Node::Container(children));
      }
      Rule::object_class => {
        let attr = pairs.next().unwrap();
        println!("object {:?}", pair);
        println!("attr {:?}", attr);
        let shape = match pair.as_str() {
          "arc" => ShapeType::Arc,
          "arrow" => ShapeType::Arrow,
          "box" => ShapeType::Box,
          &_ => !unreachable!()
        };
        ast.push(Node::Primitive(shape));
      }
      _ => {
        println!("unmatched {:?}", pair);
        parse_nodes(pair, ast);
      }
    }
  }
  ast
}

pub fn parse_node(pair: Pair<Rule>) -> Node {
  match pair.as_rule() {
    Rule::statement => parse_node(pair.into_inner().next().unwrap()),
    Rule::object_class => {
      let shape = match pair.as_str() {
        "arc" => ShapeType::Arc,
        "arrow" => ShapeType::Arrow,
        "box" => ShapeType::Box,
        &_ => !unreachable!()
      };
      Node::Primitive(shape)
    }
    _ => !unreachable!()
  }
}

pub fn parse_attrs(pair: Pair<Rule>, attrs: Vec<Attribute>) -> Vec<Attribute> {
  match pair.as_rule() {
    Rule::statement => parse_node(pair.into_inner().next().unwrap()),
    _ => !unreachable!()
  };
  attrs
}

pub fn dump_rules(level: usize, pair: Pair<Rule>) {
  for pair in pair.into_inner() {
    match pair.as_rule() {
      _ => {
        println!("{:level$} {:?}", level, pair);
        dump_rules(level + 1, pair);
      }
    }
  }
}

pub fn parse_shape(pair: Pair<Rule>, mut shape: Shape) -> Shape {
  for pair in pair.into_inner() {
    match pair.as_rule() {
      Rule::object_class => {
        shape.class = match pair.as_str() {
          "arrow" => ShapeType::Arrow,
          "box" => ShapeType::Box,
          _ => unreachable!()
        }
      }
      Rule::attribute => {
        println!("{:?}", pair);
        shape = parse_shape(pair, shape);
      }
      Rule::same_attribute => {
        shape.same = true;
      }
      Rule::size_attribute => {
        shape.fit = pair.as_str().eq("fit");
      }
      Rule::path_attribute => {
        shape.path.push(pair.into_inner().as_str().to_string());
      }
      Rule::string => {
        shape.text.push(pair.into_inner().as_str().to_string());
      }
      _ => { println!("{:?}", pair) }
    }
  }
  shape
}

pub fn shapes(pairs: Pairs<Rule>) -> Vec<Shape> {
  pairs.map(|pair|
    pair.into_inner()
  ).flat_map(|inners|
    inners.map(|inner| parse_shape(inner, Shape::default()))
  ).collect::<Vec<_>>()
}

pub fn parse_nested(pair: Pair<Rule>) {
  match pair.as_rule() {
    _ => { println!("{:?}", pair) }
  };
}