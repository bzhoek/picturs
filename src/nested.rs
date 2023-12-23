#![allow(dead_code)]

use std::error::Error;

use pest::iterators::{Pair, Pairs};
use pest::Parser;
use pest_derive::Parser;
use skia_safe::{Color, PaintStyle, Rect, Size};

use crate::nested::Node::{Container, Primitive};
use crate::skia::Canvas;

type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[derive(Parser)]
#[grammar = "nested.pest"]
pub struct NestedParser;

#[derive(PartialEq)]
#[derive(Debug)]
pub enum Node<'a> {
  Primitive(Option<&'a str>, Size, Shape<'a>),
  Container(Option<&'a str>, Size, Vec<Node<'a>>),
}

#[derive(PartialEq)]
#[derive(Debug)]
pub enum Shape<'a> {
  Line(Option<&'a str>, &'a str, &'a str),
  Rectangle(Option<&'a str>),
}

const BLOCK_PADDING: f32 = 8.;
const TEXT_PADDING: f32 = 4.;

pub fn traverse_nested<'a>(pairs: Pairs<'a, Rule>, mut ast: Vec<Node<'a>>, canvas: &mut Canvas)
                           -> (Vec<Node<'a>>, Size) {
  let mut bounds = Size::new(0., 0.);

  for pair in pairs.into_iter() {
    match pair.as_rule() {
      Rule::container => {
        let title = find_rule(&pair, Rule::inner);
        let (nodes, mut size) = traverse_nested(pair.into_inner(), vec![], canvas);
        size.height += 2. * BLOCK_PADDING;
        bounds.height += size.height;
        ast.push(Container(title, size, nodes));
      }
      Rule::line => {
        let id = find_rule(&pair, Rule::id);
        let source = find_rule(&pair, Rule::source).unwrap();
        let target = find_rule(&pair, Rule::target).unwrap();
        let size = Size::new(0., 0.);
        ast.push(Primitive(id, size, Shape::Line(id, source, target))
        );
      }
      Rule::rectangle => {
        let title = find_rule(&pair, Rule::inner);
        let id = find_rule(&pair, Rule::id);
        let height = match title {
          Some(title) => canvas.paragraph(title, (0, 0), 120.) + BLOCK_PADDING,
          None => 40.,
        };
        let size = Size::new(120., height.max(40.));
        bounds.height += size.height;
        bounds.width = bounds.width.max(size.width);
        ast.push(Primitive(id, size, Shape::Rectangle(title)));
      }
      _ => {
        println!("unmatched {:?}", pair);
        (ast, bounds) = traverse_nested(pair.into_inner(), ast, canvas);
      }
    }
  }
  (ast, bounds)
}

fn find_rule<'a>(pair: &Pair<'a, Rule>, rule: Rule) -> Option<&'a str> {
  let title = pair.clone().into_inner()
    .find(|p| p.as_rule() == rule)
    .map(|p| p.as_str());
  title
}

#[allow(dead_code)]
pub fn dump_nested(level: usize, pairs: Pairs<Rule>) {
  for pair in pairs.into_iter() {
    println!("{:level$} {:?}", level, pair);
    dump_nested(level + 1, pair.into_inner());
  }
}

pub fn parse_nested<'a>(string: &'a str, canvas: &mut Canvas) -> Result<Vec<Node<'a>>> {
  let top = nested_top(string)?;
  let (ast, _bounds) = traverse_nested(top.clone(), vec![], canvas);
  Ok(ast)
}

fn nested_top(string: &str) -> Result<Pairs<Rule>> {
  Ok(NestedParser::parse(Rule::picture, string)?)
}

fn render_nodes(ast: &Vec<Node>, canvas: &mut Canvas) -> Rect {
  let cursor = canvas.cursor;
  let mut rect = Rect::from_xywh(cursor.x, cursor.y, 0., 0.);
  for node in ast.iter() {
    match node {
      Primitive(_id, size, shape) => {
        let bounds = render_shape(shape, size, canvas);
        rect.bottom += size.height + BLOCK_PADDING;
        rect.right = bounds.right;
        canvas.cursor.y += size.height + BLOCK_PADDING;
      }
      Container(title, _, nodes) => {
        let mut bounds = render_nodes(nodes, canvas);

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

        rect.bottom += bounds.height();
        rect.right = bounds.right;
        canvas.cursor.y = bounds.bottom + BLOCK_PADDING;
      }
    }
  }
  rect
}

fn render_shape(shape: &Shape, size: &Size, canvas: &mut Canvas) -> Rect {
  let cursor = canvas.cursor;
  let rect = Rect::from_xywh(cursor.x, cursor.y, size.width, size.height);
  match shape {
    Shape::Line(_, _, _) => {}
    Shape::Rectangle(title) => {
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

#[cfg(test)]
mod tests {
  use crate::nested::Node::{Container, Primitive};
  use crate::nested::Shape;
  use crate::skia::Canvas;

  use super::*;

  static TQBF: &str = "the quick brown fox jumps over the lazy dog";

  fn parse_string(string: &str) -> (Vec<Node>, Canvas) {
    let mut canvas = Canvas::new(400, 800);
    let ast = parse_nested(string, &mut canvas).expect("Valid string");
    (ast, canvas)
  }

  #[test]
  fn single_box_untitled() -> Result<()> {
    let string = r#"box"#;
    let (ast, _canvas) = parse_string(string);
    assert_eq!(vec![
      Primitive(None, rectangle_size(), Shape::Rectangle(None)),
    ], ast);
    Ok(())
  }

  #[test]
  fn single_box_id() -> Result<()> {
    let string = r#"box.first "title""#;
    let top = nested_top(string)?;
    dump_nested(0, top);

    let mut canvas = Canvas::new(400, 800);
    let ast = parse_nested(string, &mut canvas)?;
    assert_eq!(vec![
      Primitive(Some("first"), rectangle_size(), Shape::Rectangle(Some("title"))),
    ], ast);
    Ok(())
  }

  #[test]
  fn single_box_with_title() -> Result<()> {
    let string = r#"box "title""#;
    let (ast, _canvas) = parse_string(string);

    assert_eq!(vec![
      Primitive(None, rectangle_size(), Shape::Rectangle(Some("title"))),
    ], ast);
    Ok(())
  }

  #[test]
  fn double_box() -> Result<()> {
    let string = "box
                         box";
    let (ast, _canvas) = parse_string(string);
    let size = Size::new(120., 40.);

    assert_eq!(ast, vec![
      Primitive(None, size, Shape::Rectangle(None)),
      Primitive(None, size, Shape::Rectangle(None)),
    ]);
    Ok(())
  }

  #[test]
  fn nested_box_untitled() -> Result<()> {
    let string = "box { box }";
    let (ast, _canvas) = parse_string(string);

    assert_eq!(vec![
      Container(None, container_size(), vec![
        Primitive(None, rectangle_size(), Shape::Rectangle(None))
      ])
    ], ast);
    Ok(())
  }

  fn container_size() -> Size {
    Size::new(120., 56.)
  }

  fn no_size() -> Size {
    Size::new(0., 0.)
  }

  fn rectangle_size() -> Size {
    Size::new(120., 40.)
  }

  #[test]
  fn nested_box_id() -> Result<()> {
    let string = "box.parent { box }";
    let (ast, _canvas) = parse_string(string);

    assert_eq!(vec![
      Container(None, container_size(), vec![
        Primitive(None, rectangle_size(), Shape::Rectangle(None))
      ])
    ], ast);
    Ok(())
  }

  #[test]
  fn nested_box_with_title() -> Result<()> {
    let string = r#"box "parent" { box "child" }"#;
    let (ast, _canvas) = parse_string(string);

    assert_eq!(vec![
      Container(Some("parent"), container_size(), vec![
        Primitive(None, rectangle_size(), Shape::Rectangle(Some("child")))
      ])
    ], ast);
    Ok(())
  }

  #[test]
  fn line_from_a_to_b() -> Result<()> {
    let string = r#"line from now.n to future.n"#;
    let (ast, _canvas) = parse_string(string);

    assert_eq!(ast, vec![
      Primitive(None, no_size(), Shape::Line(None, "now.n", "future.n")),
    ]);
    Ok(())
  }

  #[test]
  fn box_with_wrapping_title() -> Result<()> {
    let string = format!(r#"box "{}""#, TQBF);
    let (ast, _canvas) = parse_string(&string);
    let paragraph_size = Size::new(120., 59.);

    assert_eq!(ast, vec![
      Primitive(None, paragraph_size, Shape::Rectangle(Some(TQBF))),
    ]);
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
    let top = nested_top(string)?;
    let mut canvas = Canvas::new(400, 800);
    let (ast, bounds) = traverse_nested(top, vec![], &mut canvas);
    assert_eq!(3, ast.len());
    dbg!(&ast);
    let mut canvas = Canvas::new(400, 800);
    render_nodes(&ast, &mut canvas);
    canvas.write_png("target/extended.png");
    Ok(())
  }
}