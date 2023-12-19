use std::error::Error;

use pest::iterators::{Pair, Pairs};
use pest_derive::Parser;

pub mod nested;
pub mod skia;

pub type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[derive(Parser)]
#[grammar = "pic.pest"] // relative to project `src`
pub struct PicParser;

#[derive(PartialEq)]
#[derive(Debug)]
pub enum Node {
  Primitive(Shape, Vec<Node>),
  Container(Vec<Node>, Vec<Node>),
  Attribute(String),
  String(String),
}

#[derive(PartialEq)]
#[derive(Debug)]
pub enum Shape {
  Arc,
  Arrow,
  Box,
  Unset,
}

fn parse_next(inner: &mut Pairs<Rule>) -> Vec<Node> {
  let next = inner.next().unwrap();
  parse_nodes(next, vec![])
}

pub fn parse_nodes(pair: Pair<Rule>, mut ast: Vec<Node>) -> Vec<Node> {
  for pair in pair.into_inner() {
    match pair.as_rule() {
      Rule::container => {
        let mut inner = pair.into_inner();
        let children = parse_next(&mut inner);
        let attrs = parse_next(&mut inner);
        ast.push(Node::Container(children, attrs));
      }
      Rule::object_definition => {
        let mut inner = pair.into_inner();
        let shape = inner.next().unwrap();
        let shape = match shape.as_str() {
          "arc" => Shape::Arc,
          "arrow" => Shape::Arrow,
          "box" => Shape::Box,
          &_ => unreachable!()
        };
        let attrs = parse_next(&mut inner);
        ast.push(Node::Primitive(shape, attrs));
      }
      Rule::path_attribute => {
        ast.push(Node::Attribute(pair.as_str().to_string()))
      }
      Rule::string => {
        ast.push(Node::String(pair.into_inner().as_str().to_string()))
      }
      _ => {
        println!("unmatched {:?}", pair);
        ast = parse_nodes(pair, ast);
      }
    }
  }
  ast
}

pub fn dump_pic(level: usize, pair: Pair<Rule>) {
  for pair in pair.into_inner() {
    println!("{:level$} {:?}", level, pair);
    dump_pic(level + 1, pair);
  }
}