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
  Primitive(Option<&'a str>, Rect, Rect, Shape<'a>),
  Container(Option<&'a str>, Rect, Rect, Vec<Node<'a>>),
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
    let mut inside = Rect::from_xywh(0., 0., 400., 800.);
    inside.offset(self.offset);
    let (ast, _bounds) = self.pairs_to_nodes(top.clone(), vec![], &mut canvas, &self.offset);
    self.nodes = ast;
    top
  }

  pub fn pairs_to_nodes<'a>(&self, pairs: Pairs<'a, Rule>, mut ast: Vec<Node<'a>>, canvas: &mut Canvas, offset: &Point)
                            -> (Vec<Node<'a>>, Rect) {
    let mut bounds = Rect::from_xywh(offset.x, offset.y, 0., 0.);

    for pair in pairs.into_iter() {
      match pair.as_rule() {
        Rule::container => {
          let title = rule_to_string(&pair, Rule::inner);
          let mut used = Rect::from_xywh(bounds.left, bounds.bottom, 0., 0.);
          let mut inset = Point::new(used.left, used.bottom);
          inset.offset((BLOCK_PADDING, BLOCK_PADDING));
          let (nodes, inner)
            = self.pairs_to_nodes(pair.into_inner(), vec![], canvas, &inset);
          used.bottom = inner.bottom + BLOCK_PADDING;
          used.right = inner.right + 2. * BLOCK_PADDING;

          if let Some(title) = title {
            let text_inset = inner.with_inset((TEXT_PADDING, TEXT_PADDING));
            let down = canvas.paragraph(title, (0., 0.), text_inset.width());
            used.bottom = inner.bottom + down + TEXT_PADDING;
          }

          let mut rect = used;
          rect.bottom += BLOCK_PADDING;
          ast.push(Container(title, rect, used, nodes));

          bounds.right = bounds.right.max(rect.right);
          bounds.bottom = rect.bottom;
        }
        Rule::line => {
          let id = rule_to_string(&pair, Rule::id);
          let source = rule_to_string(&pair, Rule::source).unwrap();
          let target = rule_to_string(&pair, Rule::target).unwrap();
          let rect = Rect::from_xywh(bounds.left, bounds.bottom, 0., 0.);
          ast.push(Primitive(id, rect, rect, Shape::Line(id, source, target))
          );
        }
        Rule::rectangle => {
          let title = rule_to_string(&pair, Rule::inner);
          let id = rule_to_string(&pair, Rule::id);
          let location = rule_to_location(&pair, Rule::location);
          let height = match title {
            Some(title) => canvas.paragraph(title, (0., 0.), 120. - 2. * TEXT_PADDING),
            None => 40.,
          };

          let mut used = Rect::from_xywh(bounds.left, bounds.bottom, 120., height.max(40.));
          used.bottom += BLOCK_PADDING;
          let mut rect = used;
          rect.bottom += BLOCK_PADDING;
          ast.push(Primitive(id, rect, used, Shape::Rectangle(title, location)));

          bounds.right = bounds.right.max(rect.right);
          bounds.bottom = rect.bottom
        }
        _ => {
          println!("unmatched {:?}", pair);
          let inset = Point::new(bounds.left, bounds.bottom);
          (ast, bounds) = self.pairs_to_nodes(pair.into_inner(), ast, canvas, &inset);
        }
      }
    }
    (ast, bounds)
  }

  fn find_node<'a>(&'a self, id: &str) -> Option<&Node<'a>> {
    self.search_nodes(&self.nodes, id)
  }

  fn search_nodes<'a>(&'a self, nodes: &'a [Node], node_id: &str) -> Option<&Node<'a>> {
    for node in nodes.iter() {
      match node {
        Primitive(id, _, _, _) => {
          if let Some(id) = id {
            if id == &node_id {
              return Some(node);
            }
          }
        }
        Container(_, _, _, nodes) => {
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
    self.render_nodes(&self.nodes, &mut canvas);
    canvas.write_png(filepath);
  }

  fn render_nodes(&self, nodes: &[Node], canvas: &mut Canvas) {
    for node in nodes.iter() {
      match node {
        Container(title, _rect, used, nodes) => {
          self.render_nodes(nodes, canvas);

          if let Some(title) = title {
            canvas.paint.set_style(PaintStyle::Fill);
            canvas.paint.set_color(Color::BLACK);
            let inset = used.with_inset((TEXT_PADDING, TEXT_PADDING));
            let origin = (inset.left, inset.bottom - 16.);
            canvas.paragraph(title, origin, inset.width());
          }

          canvas.paint.set_style(PaintStyle::Stroke);
          canvas.paint.set_color(Color::RED);
          canvas.rectangle(used);
        }
        Primitive(_id, rect, used, shape) => {
          self.render_shape(shape, rect, used, canvas);
        }
      }
    }
  }

  fn render_shape(&self, shape: &Shape, rect: &Rect, used: &Rect, canvas: &mut Canvas) -> Rect {
    let mut moved = *used;

    let mut adjust = |other: &Rect, distance: &Distance| {
      moved.top = other.top;
      moved.bottom = moved.top + rect.height();
      moved.left = other.right + distance.pixels();
      moved.right = moved.left + rect.width()
    };

    match shape {
      Shape::Line(_, _, _) => {}
      Shape::Rectangle(title, location) => {
        if let Some(location) = location {
          let (_compass, distance, edge) = location;
          if let Some(node) = self.find_node(edge.0) {
            match node {
              Primitive(_, other, _, _) => adjust(other, distance),
              Container(_, other, _, _) => adjust(other, distance)
            };
          };
        }

        canvas.paint.set_style(PaintStyle::Stroke);
        canvas.paint.set_color(Color::BLUE);
        canvas.rectangle(&moved);

        if let Some(title) = title {
          canvas.paint.set_style(PaintStyle::Fill);
          canvas.paint.set_color(Color::BLACK);
          let inset = moved.with_inset((TEXT_PADDING, TEXT_PADDING));
          let origin = (inset.left, moved.top);
          canvas.paragraph(title, origin, inset.width());
        }
      }
    }
    moved
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

  use Shape::Rectangle;

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
      Primitive(None,
                Rect::from_xywh(0., 0., 120., 56.),
                Rect::from_xywh(0., 0., 120., 48.),
                Rectangle(None, None)),
    ], diagram.nodes);
  }

  #[test]
  fn single_box_id() {
    let string = r#"box.first "title""#;
    let diagram = parse_string(string);

    assert_eq!(vec![
      Primitive(Some("first"),
                Rect::from_xywh(0., 0., 120., 56.),
                Rect::from_xywh(0., 0., 120., 48.),
                Rectangle(Some("title"), None)),
    ], diagram.nodes);
  }

  #[test]
  fn single_box_with_title() {
    let string = r#"box "title""#;
    let diagram = parse_string(string);

    assert_eq!(vec![
      Primitive(None,
                Rect::from_xywh(0., 0., 120., 56.),
                Rect::from_xywh(0., 0., 120., 48.),
                Rectangle(Some("title"), None)),
    ], diagram.nodes);
  }

  #[test]
  fn double_box() {
    let string = "box
                         box";
    let diagram = parse_string(string);

    assert_eq!(vec![
      Primitive(None,
                Rect::from_xywh(0., 0., 120., 56.),
                Rect::from_xywh(0., 0., 120., 48.),
                Rectangle(None, None)),
      Primitive(None,
                Rect::from_xywh(0., 56., 120., 56.),
                Rect::from_xywh(0., 56., 120., 48.),
                Rectangle(None, None)),
    ], diagram.nodes);
  }

  #[test]
  fn nested_box_untitled() {
    let string = "box { box }";
    let diagram = parse_string(string);

    assert_eq!(vec![
      Container(None,
                Rect::from_xywh(0., 0., 144., 80.),
                Rect::from_xywh(0., 0., 144., 72.),
                vec![
                  Primitive(None,
                            Rect::from_xywh(8., 8., 120., 56.),
                            Rect::from_xywh(8., 8., 120., 48.),
                            Rectangle(None, None))
                ])
    ], diagram.nodes);
  }

  #[test]
  fn nested_box_id() {
    let string = "box.parent { box }";
    let diagram = parse_string(string);

    assert_eq!(vec![
      Container(None,
                Rect::from_xywh(0., 0., 144., 80.),
                Rect::from_xywh(0., 0., 144., 72.),
                vec![
                  Primitive(None,
                            Rect::from_xywh(8., 8., 120., 56.),
                            Rect::from_xywh(8., 8., 120., 48.),
                            Rectangle(None, None))
                ])
    ], diagram.nodes);
  }

  #[test]
  fn nested_box_with_title() {
    let string = r#"box "parent" { box "child" }"#;
    let diagram = parse_string(string);
    let container_rect = Rect::from_xywh(0., 0., 136., 77.);

    assert_eq!(vec![
      Container(Some("parent"),
                Rect::from_xywh(0., 0., 144., 93.),
                Rect::from_xywh(0., 0., 144., 85.),
                vec![
                  Primitive(None,
                            Rect::from_xywh(8., 8., 120., 56.),
                            Rect::from_xywh(8., 8., 120., 48.),
                            Rectangle(Some("child"), None))
                ])
    ], diagram.nodes);
  }

  #[test]
  fn line_from_a_to_b() {
    let string = r#"line from now.n to future.n"#;
    let diagram = parse_string(string);

    assert_eq!(vec![
      Primitive(None,
                zero_rect(),
                zero_rect(),
                Shape::Line(None, "now.n", "future.n")),
    ], diagram.nodes);
  }

  #[test]
  fn box_with_wrapping_title() {
    let string = format!(r#"box "{}""#, TQBF);
    let diagram = parse_string(&string);
    let paragraph1_rect = Rect::from_xywh(0., 0., 120., 84.);
    let paragraph2_rect = Rect::from_xywh(0., 0., 120., 76.);

    assert_eq!(vec![
      Primitive(None,
                paragraph1_rect,
                paragraph2_rect,
                Rectangle(Some(TQBF), None)),
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
      Primitive(_, rect, _, _) => { rect }
      Container(_, rect, _, _) => { rect }
    };
    let rect = Rect { left: 32., top: 32., right: 152., bottom: 99. };
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