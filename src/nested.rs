#![allow(dead_code)]

use std::error::Error;

use pest::iterators::{Pair, Pairs};
use pest::Parser;
use pest_derive::Parser;
use skia_safe::{Color, PaintStyle, Point, Rect, Size};

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

#[derive(PartialEq)]
#[derive(Debug)]
pub enum Shape<'a> {
  Line(Option<&'a str>, &'a str, &'a str),
  Rectangle(Option<&'a str>, Option<(&'a str, &'a str, (&'a str, &'a str))>),
}

#[derive(Default)]
pub struct Diagram<'a> {
  nodes: Vec<Node<'a>>,
  offset: Point,
}

impl<'i> Diagram<'i> {
  pub fn set_offset(&mut self, offset: impl Into<Point>) {
    self.offset = offset.into();
  }

  pub fn parse_string(&mut self, string: &'i str) {
    let top = NestedParser::parse(Rule::picture, string).unwrap();
    let mut canvas = Canvas::new(400, 800);
    canvas.cursor = self.offset;
    let (ast, _bounds) = self.pairs_to_nodes(top.clone(), vec![], &mut canvas);
    self.nodes = ast;
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
          rect.bottom += 2. * BLOCK_PADDING;
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

          let size = Size::new(120., height.max(40.));
          let rect = Rect::from_xywh(cursor.x, cursor.y, size.width, size.height);

          ast.push(Primitive(id, rect, Shape::Rectangle(title, location)));
          outer_rect.right = outer_rect.right.max(rect.right);
          outer_rect.bottom = rect.bottom;
          canvas.cursor.y = rect.bottom;
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
        Primitive(_id, size, shape) => {
          let bounds = self.render_shape(shape, size, canvas);
          outer_rect.bottom += size.height() + BLOCK_PADDING;
          outer_rect.right = bounds.right;
          canvas.cursor.y += size.height() + BLOCK_PADDING;
        }
        Container(title, _, nodes) => {
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

          outer_rect.bottom += bounds.height();
          outer_rect.right = bounds.right;
          canvas.cursor.y = bounds.bottom + BLOCK_PADDING;
        }
      }
    }
    outer_rect
  }

  fn render_shape(&self, shape: &Shape, size: &Rect, canvas: &mut Canvas) -> Rect {
    let cursor = canvas.cursor;
    let rect = Rect::from_xywh(cursor.x, cursor.y, size.width(), size.height());
    match shape {
      Shape::Line(_, _, _) => {}
      Shape::Rectangle(title, location) => {
        if let Some(_location) = location {
          // let (id, compass, edge) = location;
          // let (x, y) = canvas.find_edge(id, compass, edge);
          // canvas.cursor = (x, y);
        }
        canvas.paint.set_style(PaintStyle::Stroke);
        canvas.paint.set_color(Color::BLUE);
        canvas.rectangle(&rect);
        if let Some(title) = title {
          canvas.paint.set_style(PaintStyle::Fill);
          canvas.paint.set_color(Color::BLACK);
          canvas.paragraph(title, (rect.left + TEXT_PADDING, rect.top), rect.width() - 2. * TEXT_PADDING);
        }
      }
    }
    rect
  }
}

const BLOCK_PADDING: f32 = 8.;
const TEXT_PADDING: f32 = 4.;

fn rule_to_location<'a>(pair: &Pair<'a, Rule>, rule: Rule) -> Option<(&'a str, &'a str, (&'a str, &'a str))> {
  find_rule(pair, rule)
    .map(|p| (
      rule_to_string(&p, Rule::compass).unwrap(),
      rule_to_string(&p, Rule::distance).unwrap(),
      rule_to_edge(&p, Rule::edge).unwrap(),
    ))
}

fn rule_to_edge<'a>(pair: &Pair<'a, Rule>, rule: Rule) -> Option<(&'a str, &'a str)> {
  find_rule(pair, rule)
    .map(|p| (
      rule_to_string(&p, Rule::id).unwrap(),
      rule_to_string(&p, Rule::compass).unwrap(),
    ))
}

fn rule_to_string<'a>(pair: &Pair<'a, Rule>, rule: Rule) -> Option<&'a str> {
  find_rule(pair, rule).map(|p| p.as_str())
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
  use crate::nested::Node::{Container, Primitive};
  use crate::nested::Shape;

  use super::*;

  static TQBF: &str = "the quick brown fox jumps over the lazy dog";

  fn container_size() -> Rect {
    Rect::from_xywh(0., 56., 120., 40.)
  }

  fn no_size() -> Rect {
    Rect::from_xywh(0., 0., 0., 0.)
  }

  fn rectangle_size() -> Rect {
    Rect::from_xywh(0., 0., 120., 40.)
  }

  fn parse_string(string: &str) -> Diagram {
    let mut diagram = Diagram::default();
    diagram.parse_string(string);
    diagram
  }

  #[test]
  fn single_box_untitled() -> Result<()> {
    let string = r#"box"#;
    let diagram = parse_string(string);
    assert_eq!(vec![
      Primitive(None, rectangle_size(), Shape::Rectangle(None, None)),
    ], diagram.nodes);
    Ok(())
  }

  #[test]
  fn single_box_id() -> Result<()> {
    let string = r#"box.first "title""#;
    let diagram = parse_string(string);
    assert_eq!(vec![
      Primitive(Some("first"), rectangle_size(), Shape::Rectangle(Some("title"), None)),
    ], diagram.nodes);
    Ok(())
  }

  #[test]
  fn single_box_with_title() -> Result<()> {
    let string = r#"box "title""#;
    let diagram = parse_string(string);

    assert_eq!(vec![
      Primitive(None, rectangle_size(), Shape::Rectangle(Some("title"), None)),
    ], diagram.nodes);
    Ok(())
  }

  #[test]
  fn double_box() -> Result<()> {
    let string = "box
                         box";
    let diagram = parse_string(string);

    assert_eq!(vec![
      Primitive(None, rectangle_size(), Shape::Rectangle(None, None)),
      Primitive(None, rectangle_size(), Shape::Rectangle(None, None)),
    ], diagram.nodes);
    Ok(())
  }

  #[test]
  fn nested_box_untitled() -> Result<()> {
    let string = "box { box }";
    let diagram = parse_string(string);

    assert_eq!(vec![
      Container(None, container_size(), vec![
        Primitive(None, rectangle_size(), Shape::Rectangle(None, None))
      ])
    ], diagram.nodes);
    Ok(())
  }

  #[test]
  fn nested_box_id() -> Result<()> {
    let string = "box.parent { box }";
    let diagram = parse_string(string);

    assert_eq!(vec![
      Container(None, container_size(), vec![
        Primitive(None, rectangle_size(), Shape::Rectangle(None, None))
      ])
    ], diagram.nodes);
    Ok(())
  }

  #[test]
  fn nested_box_with_title() -> Result<()> {
    let string = r#"box "parent" { box "child" }"#;
    let diagram = parse_string(string);

    assert_eq!(vec![
      Container(Some("parent"), container_size(), vec![
        Primitive(None, rectangle_size(), Shape::Rectangle(Some("child"), None))
      ])
    ], diagram.nodes);
    Ok(())
  }

  #[test]
  fn line_from_a_to_b() -> Result<()> {
    let string = r#"line from now.n to future.n"#;
    let diagram = parse_string(string);

    assert_eq!(vec![
      Primitive(None, no_size(), Shape::Line(None, "now.n", "future.n")),
    ], diagram.nodes);
    Ok(())
  }

  #[test]
  fn box_with_wrapping_title() -> Result<()> {
    let string = format!(r#"box "{}""#, TQBF);
    let diagram = parse_string(&string);
    let paragraph_size = Rect::from_xywh(0., 0., 120., 76.);

    assert_eq!(vec![
      Primitive(None, paragraph_size, Shape::Rectangle(Some(TQBF), None)),
    ], diagram.nodes);
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
    let mut diagram = Diagram::default();
    diagram.parse_string(string);
    diagram.set_offset((32., 32.));
    dbg!(&diagram.nodes);
    diagram.render(400, 800, "target/extended.png");
    Ok(())
  }


  #[test]
  fn side_by_side() -> Result<()> {
    let string =
      r#"box.left "This goes to the left hand side"
      box.right "While this goes to the right hand side" @nw 1cm from left.ne
      "#;
    let mut diagram = Diagram::default();
    diagram.parse_string(string);
    diagram.set_offset((32., 32.));
    assert_eq!(2, diagram.nodes.len());
    dbg!(&diagram.nodes);
    let left = diagram.find_node("left").unwrap();
    let rect = match left {
      Primitive(_, rect, _) => { rect }
      Container(_, rect, _) => { rect }
    };
    assert_eq!(59., rect.height());

    diagram.render(400, 800, "target/side_by_side.png");
    Ok(())
  }
}