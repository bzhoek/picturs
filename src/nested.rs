#![allow(dead_code)]

use std::error::Error;

use pest::iterators::{Pair, Pairs};
use pest::Parser;
use pest_derive::Parser;
use skia_safe::{Color, PaintStyle, Point, Rect};
use crate::Distance;

use crate::nested::Node::{Container, Primitive};
use crate::skia::Canvas;

type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[derive(Parser)]
#[grammar = "nested.pest"]
pub struct NestedParser;

#[derive(PartialEq)]
#[derive(Debug)]
pub enum Node<'a> {
  Primitive(Option<&'a str>, Rect, Shape<'a>),
  Container(Option<&'a str>, Rect, Vec<Node<'a>>),
}

#[derive(Debug)]
#[derive(PartialEq)]
pub enum Shape<'a> {
  Line(Option<&'a str>, &'a str, &'a str),
  Rectangle(Option<&'a str>, Option<(&'a str, Distance, (&'a str, &'a str))>),
}

#[derive(Default)]
pub struct Diagram<'a> {
  nodes: Vec<Node<'a>>,
  offset: Point,
}

impl<'i> Diagram<'i> {
  pub fn offset(offset: impl Into<Point>) -> Self {
    Self {
      nodes: vec![],
      offset: offset.into(),
    }
  }

  pub fn parse_string(&mut self, string: &'i str) -> Pairs<'i, Rule> {
    let top = NestedParser::parse(Rule::picture, string).unwrap();
    let mut canvas = Canvas::new(400, 800);
    canvas.cursor = self.offset;
    let (ast, _bounds) = self.pairs_to_nodes(top.clone(), vec![], &mut canvas);
    self.nodes = ast;
    top
  }

  pub fn pairs_to_nodes<'a>(&self, pairs: Pairs<'a, Rule>, mut ast: Vec<Node<'a>>, canvas: &mut Canvas)
                            -> (Vec<Node<'a>>, Rect) {
    let cursor = canvas.cursor;
    let mut outer_rect = Rect::from_xywh(cursor.x, cursor.y, 0., 0.);

    for pair in pairs.into_iter() {
      match pair.as_rule() {
        Rule::container => {
          let title = rule_to_string(&pair, Rule::inner);
          let (nodes, mut rect) = self.pairs_to_nodes(pair.into_inner(), vec![], canvas);
          canvas.cursor.y -= BLOCK_PADDING;

          rect.bottom += 2. * BLOCK_PADDING;
          rect.right += 2. * BLOCK_PADDING;

          if let Some(title) = title {
            let down = canvas.paragraph(title, (rect.left + TEXT_PADDING, rect.bottom - TEXT_PADDING), rect.width() - 2. * TEXT_PADDING);
            rect.bottom += down + TEXT_PADDING;
          }

          ast.push(Container(title, rect, nodes));

          outer_rect.right = outer_rect.right.max(rect.right);
          outer_rect.bottom = rect.bottom;
          canvas.cursor.y = rect.bottom;
        }
        Rule::line => {
          let id = rule_to_string(&pair, Rule::id);
          let source = rule_to_string(&pair, Rule::source).unwrap();
          let target = rule_to_string(&pair, Rule::target).unwrap();
          let rect = Rect::from_xywh(cursor.x, cursor.y, 0., 0.);
          ast.push(Primitive(id, rect, Shape::Line(id, source, target))
          );
        }
        Rule::rectangle => {
          let title = rule_to_string(&pair, Rule::inner);
          let id = rule_to_string(&pair, Rule::id);
          let location = rule_to_location(&pair, Rule::location);
          let height = match title {
            Some(title) => canvas.paragraph(title, (TEXT_PADDING, 0.), 120. - 2. * TEXT_PADDING) + BLOCK_PADDING,
            None => 40.,
          };

          let cursor = canvas.cursor;
          let rect = Rect::from_xywh(cursor.x, cursor.y, 120., height.max(40.));

          ast.push(Primitive(id, rect, Shape::Rectangle(title, location)));
          outer_rect.right = outer_rect.right.max(rect.right);
          outer_rect.bottom = rect.bottom;
          canvas.cursor.y = rect.bottom + BLOCK_PADDING
        }
        _ => {
          println!("unmatched {:?}", pair);
          (ast, outer_rect) = self.pairs_to_nodes(pair.into_inner(), ast, canvas);
        }
      }
    }
    (ast, outer_rect)
  }

  fn find_node<'a>(&'a self, id: &str) -> Option<&Node<'a>> {
    self.search_nodes(&self.nodes, id)
  }

  fn search_nodes<'a>(&'a self, nodes: &'a [Node], node_id: &str) -> Option<&Node<'a>> {
    for node in nodes.iter() {
      match node {
        Primitive(id, _, _) => {
          if let Some(id) = id {
            if id == &node_id {
              return Some(node);
            }
          }
        }
        Container(_, _, nodes) => {
          if let Some(node) = self.search_nodes(nodes, node_id) {
            return Some(node);
          };
        }
      }
    }
    None
  }

  pub fn render(&self, width: i32, height: i32, filepath: &str) {
    let mut canvas = Canvas::new(width, height);
    canvas.cursor = self.offset;
    let _bounds = self.render_nodes(&self.nodes, &mut canvas);
    canvas.write_png(filepath);
  }

  fn render_nodes(&self, nodes: &[Node], canvas: &mut Canvas) -> Rect {
    let cursor = canvas.cursor;
    let mut outer_rect = Rect::from_xywh(cursor.x, cursor.y, 0., 0.);
    for node in nodes.iter() {
      match node {
        Container(title, _rect, nodes) => {
          let mut bounds = self.render_nodes(nodes, canvas);

          bounds.top -= BLOCK_PADDING;
          bounds.left -= BLOCK_PADDING;
          bounds.right += BLOCK_PADDING;

          if let Some(title) = title {
            canvas.paint.set_style(PaintStyle::Fill);
            canvas.paint.set_color(Color::BLACK);
            let down = canvas.paragraph(title, (bounds.left + TEXT_PADDING, bounds.bottom - TEXT_PADDING), bounds.width() - 2. * TEXT_PADDING);
            bounds.bottom += down + TEXT_PADDING;
          }

          canvas.paint.set_style(PaintStyle::Stroke);
          canvas.paint.set_color(Color::RED);
          canvas.rectangle(&bounds);

          outer_rect.right = bounds.right;
          outer_rect.bottom += bounds.height();
          canvas.cursor.y = bounds.bottom + BLOCK_PADDING;
        }
        Primitive(_id, rect, shape) => {
          self.render_shape(shape, rect, canvas);
          outer_rect.right = rect.right;
          outer_rect.bottom += rect.height() + BLOCK_PADDING;
          canvas.cursor.y = outer_rect.bottom
        }
      }
    }
    outer_rect
  }

  fn render_shape(&self, shape: &Shape, rect: &Rect, canvas: &mut Canvas) -> Rect {
    let mut adjusted = *rect;

    let mut adjust = |other: &Rect, distance: &Distance| {
      adjusted.top = other.top;
      adjusted.bottom = adjusted.top + rect.height();
      adjusted.left = other.right + distance.pixels();
      adjusted.right = adjusted.left + rect.width()
    };

    match shape {
      Shape::Line(_, _, _) => {}
      Shape::Rectangle(title, location) => {
        if let Some(location) = location {
          let (_compass, distance, edge) = location;
          if let Some(node) = self.find_node(edge.0) {
            match node {
              Primitive(_, other, _) => adjust(other, distance),
              Container(_, other, _) => adjust(other, distance)
            };
          };
        }
        canvas.paint.set_style(PaintStyle::Stroke);
        canvas.paint.set_color(Color::BLUE);
        canvas.rectangle(&adjusted);
        if let Some(title) = title {
          canvas.paint.set_style(PaintStyle::Fill);
          canvas.paint.set_color(Color::BLACK);
          canvas.paragraph(title, (adjusted.left + TEXT_PADDING, adjusted.top), adjusted.width() - 2. * TEXT_PADDING);
        }
      }
    }
    adjusted
  }
}

const BLOCK_PADDING: f32 = 8.;
const TEXT_PADDING: f32 = 4.;

fn rule_to_location<'a>(pair: &Pair<'a, Rule>, rule: Rule)
                        -> Option<(&'a str, Distance, (&'a str, &'a str))> {
  find_rule(pair, rule)
    .map(|p| (
      rule_to_string(&p, Rule::compass).unwrap(),
      rule_to_distance(&p, Rule::distance).unwrap(),
      rule_to_edge(&p, Rule::edge).unwrap(),
    ))
}

fn rule_to_distance(pair: &Pair<Rule>, rule: Rule) -> Option<Distance> {
  find_rule(pair, rule)
    .map(|p| {
      let length = find_rule(&p, Rule::length)
        .and_then(|p| p.as_str().parse::<usize>().ok())
        .unwrap();
      let unit = rule_to_string(&p, Rule::unit)
        .unwrap();
      Distance::new(length as f32, unit.to_owned())
    })
}

fn rule_to_edge<'a>(pair: &Pair<'a, Rule>, rule: Rule) -> Option<(&'a str, &'a str)> {
  find_rule(pair, rule)
    .map(|p| (
      rule_to_string(&p, Rule::id).unwrap(),
      rule_to_string(&p, Rule::compass).unwrap(),
    ))
}

fn rule_to_string<'a>(pair: &Pair<'a, Rule>, rule: Rule) -> Option<&'a str> {
  find_rule(pair, rule)
    .map(|p| p.as_str())
}

fn find_rule<'a>(pair: &Pair<'a, Rule>, rule: Rule) -> Option<Pair<'a, Rule>> {
  pair.clone().into_inner()
    .find(|p| p.as_rule() == rule)
}

#[allow(dead_code)]
pub fn dump_nested(level: usize, pairs: Pairs<Rule>) {
  for pair in pairs.into_iter() {
    println!("{:level$} {:?}", level, pair);
    dump_nested(level + 1, pair.into_inner());
  }
}

#[cfg(test)]
mod tests {
  use std::fs;
  use std::path::Path;
  use std::process::Command;

  use crate::nested::Node::{Container, Primitive};
  use crate::nested::Shape;

  use super::*;

  static TQBF: &str = "the quick brown fox jumps over the lazy dog";

  fn container_rect() -> Rect {
    Rect::from_xywh(0., 0., 136., 56.)
  }

  fn zero_rect() -> Rect {
    Rect::from_xywh(0., 0., 0., 0.)
  }

  fn rectangle_rect() -> Rect {
    Rect::from_xywh(0., 0., 120., 40.)
  }

  fn parse_string(string: &str) -> Diagram {
    let mut diagram = Diagram::default();
    diagram.parse_string(string);
    diagram
  }

  #[test]
  fn single_box_untitled() {
    let string = r#"box"#;
    let diagram = parse_string(string);

    assert_eq!(vec![
      Primitive(None, rectangle_rect(), Shape::Rectangle(None, None)),
    ], diagram.nodes);
  }

  #[test]
  fn single_box_id() {
    let string = r#"box.first "title""#;
    let diagram = parse_string(string);

    assert_eq!(vec![
      Primitive(Some("first"), rectangle_rect(), Shape::Rectangle(Some("title"), None)),
    ], diagram.nodes);
  }

  #[test]
  fn single_box_with_title() {
    let string = r#"box "title""#;
    let diagram = parse_string(string);

    assert_eq!(vec![
      Primitive(None, rectangle_rect(), Shape::Rectangle(Some("title"), None)),
    ], diagram.nodes);
  }

  #[test]
  fn double_box() {
    let string = "box
                         box";
    let diagram = parse_string(string);

    let second_rect = Rect::from_xywh(0., 48., 120., 40.);
    assert_eq!(vec![
      Primitive(None, rectangle_rect(), Shape::Rectangle(None, None)),
      Primitive(None, second_rect, Shape::Rectangle(None, None)),
    ], diagram.nodes);
  }

  #[test]
  fn nested_box_untitled() {
    let string = "box { box }";
    let diagram = parse_string(string);

    assert_eq!(vec![
      Container(None, container_rect(), vec![
        Primitive(None, rectangle_rect(), Shape::Rectangle(None, None))
      ])
    ], diagram.nodes);
  }

  #[test]
  fn nested_box_id() {
    let string = "box.parent { box }";
    let diagram = parse_string(string);

    assert_eq!(vec![
      Container(None, container_rect(), vec![
        Primitive(None, rectangle_rect(), Shape::Rectangle(None, None))
      ])
    ], diagram.nodes);
  }

  #[test]
  fn nested_box_with_title() {
    let string = r#"box "parent" { box "child" }"#;
    let diagram = parse_string(string);
    let container_rect = Rect::from_xywh(0., 0., 136., 77.);

    assert_eq!(vec![
      Container(Some("parent"), container_rect, vec![
        Primitive(None, rectangle_rect(), Shape::Rectangle(Some("child"), None))
      ])
    ], diagram.nodes);
  }

  #[test]
  fn line_from_a_to_b() {
    let string = r#"line from now.n to future.n"#;
    let diagram = parse_string(string);

    assert_eq!(vec![
      Primitive(None, zero_rect(), Shape::Line(None, "now.n", "future.n")),
    ], diagram.nodes);
  }

  #[test]
  fn box_with_wrapping_title() {
    let string = format!(r#"box "{}""#, TQBF);
    let diagram = parse_string(&string);
    let paragraph_rect = Rect::from_xywh(0., 0., 120., 76.);

    assert_eq!(vec![
      Primitive(None, paragraph_rect, Shape::Rectangle(Some(TQBF), None)),
    ], diagram.nodes);
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
    let mut diagram = Diagram::offset((32., 32.));
    diagram.parse_string(string);
    dbg!(&diagram.nodes);
    assert_visual(diagram, "target/extended")?;
    Ok(())
  }

  #[test]
  fn side_by_side() -> Result<()> {
    let string =
      r#"box.left "This goes to the left hand side"
      box.right "While this goes to the right hand side" @nw 2cm from left.ne
      "#;
    let mut diagram = Diagram::offset((32., 32.));
    let top = diagram.parse_string(string);
    dump_nested(0, top);
    assert_eq!(2, diagram.nodes.len());
    dbg!(&diagram.nodes);
    let left = diagram.find_node("left").unwrap();
    let left_rect = match left {
      Primitive(_, rect, _) => { rect }
      Container(_, rect, _) => { rect }
    };
    let rect = Rect::from_ltrb(32., 32., 152., 91.);
    assert_eq!(&rect, left_rect);

    assert_visual(diagram, "target/side_by_side")?;
    Ok(())
  }

  fn assert_visual(diagram: Diagram, prefix: &str) -> Result<()> {
    let ref_file = format!("{}.png", prefix);
    let last_file = format!("{}-last.png", prefix);
    diagram.render(400, 800, &*last_file);
    if !Path::new(&ref_file).exists() {
      fs::rename(last_file, ref_file)?;
    } else {
      let diff_file = format!("{}-diff.png", prefix);
      let output = Command::new("compare")
        .arg("-metric")
        .arg("rmse")
        .arg(&last_file)
        .arg(ref_file)
        .arg(&diff_file)
        .output()?;
      assert!(output.status.success());
      fs::remove_file(last_file)?;
      fs::remove_file(diff_file)?;
    }
    Ok(())
  }
}