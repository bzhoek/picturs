use std::f32::consts::PI;
use std::ops::{Sub};

use log::warn;
use skia_safe::textlayout::TextAlign;
use skia_safe::{Color, PaintStyle, Point, Rect};

use crate::diagram::attributes::Attributes;
use crate::diagram::parser::TEXT_PADDING;
use crate::diagram::types::Node::{Closed, Container, Open, Primitive};
use crate::diagram::types::{Caption, Ending, Endings, Node, Paragraph, Radius, Shape};
use crate::skia::Canvas;
use crate::skia::Effect::Solid;

pub struct Renderer {}

impl Renderer {
  pub fn render_to_canvas(canvas: &mut Canvas, nodes: &[Node]) {
    for node in nodes.iter() {
      canvas.paint.set_stroke_width(1.0);

      match node {
        Container(Attributes::Closed { radius, title, thickness, effect, stroke, .. }, used, nodes) => {
          Self::render_to_canvas(canvas, nodes);

          if let Some(title) = title {
            canvas.fill_with(Color::BLACK);
            let inset = used.with_inset((TEXT_PADDING, TEXT_PADDING));
            let origin = (inset.left, inset.bottom - 16.);
            canvas.draw_paragraph(title, origin, inset.width());
          }

          if thickness > &0. {
            canvas.stroke_with(*thickness, *stroke, effect);
            canvas.rectangle(used, *radius);
          }
        }
        Primitive(common, shape) => {
          let used = Self::align_rect(&common.used, common.thickness);
          Self::render_shape(canvas, &used, &common.stroke, shape, &common.thickness);
        }
        Open(Attributes::Open { thickness, stroke, .. }, used, shape) => {
          let used = Self::align_rect(used, *thickness);
          Self::render_shape(canvas, &used, stroke, shape, thickness);
        }
        Closed(Attributes::Closed { radius, thickness, effect, stroke, fill, text, location, endings, .. }, used, paragraph, shape) => {
          let used = Self::align_rect(used, *thickness);

          canvas.stroke_with(*thickness, *stroke, effect);
          match shape {
            Shape::Rectangle => canvas.rectangle(&used, *radius),
            Shape::Circle => canvas.circle(&used.center(), used.width() / 2.),
            Shape::Ellipse => canvas.ellipse(&used),
            Shape::File => canvas.file(&used),
            Shape::Oval => canvas.oval(&used),
            Shape::Cylinder => canvas.cylinder(&used),
            _ => {}
          }

          if let (Some(_endings), Some((my, displacements, _))) = (endings, location) {
            let mut points: Vec<Point> = vec![];
            let mut point = my.edge_point(&used);
            canvas.move_to(point.x, point.y);
            points.push(point);
            for movement in displacements.iter() {
              point = point.sub(movement.offset());
              points.push(point);
              canvas.line_to(point.x, point.y);
            }

            let start = points.first().unwrap();
            let end = points.get(1).unwrap();
            Self::draw_ending(&_endings.start, start, end, canvas);
            let start = points.get(points.len() - 2).unwrap();
            let end = points.last().unwrap();
            Self::draw_ending(&_endings.end, end, start, canvas);
            canvas.stroke();
          }

          canvas.fill_with(*fill);
          match shape {
            Shape::Rectangle => canvas.rectangle(&used, *radius),
            Shape::Circle => canvas.circle(&used.center(), used.width() / 2.),
            Shape::Ellipse => canvas.ellipse(&used),
            Shape::File => canvas.file(&used),
            Shape::Oval => canvas.oval(&used),
            Shape::Cylinder => canvas.cylinder(&used),
            _ => {}
          }

          match shape {
            Shape::Cylinder => {
              let rect = Rect::from_xywh(used.left, used.top + used.height() / 3., used.width(), used.height() * 0.666);
              Self::paint_paragraph(canvas, &rect, text, paragraph);
            }
            _ => Self::paint_paragraph(canvas, &used, text, paragraph)
          }
        }
        Node::Font(font) => canvas.font = font.clone(),
        Node::Move(_used) => {}
        _ => warn!("Cannot render: {:?}", node),
      }
    }
  }

  fn render_shape(canvas: &mut Canvas, used: &Rect, color: &Color, shape: &Shape, thickness: &f32) {
    match shape {
      Shape::Path(points, caption) => {
        canvas.paint.set_style(PaintStyle::Stroke);
        canvas.paint.set_color(*color);
        let mut iter = points.iter();
        let start = iter.next().unwrap();
        canvas.move_to(start.x, start.y);
        for point in iter {
          canvas.line_to(point.x, point.y);
        }
        canvas.stroke();

        Self::draw_caption_in(caption, used, canvas);
      }
      Shape::Sline(points, caption, endings) => {
        canvas.stroke_with(*thickness, *color, &Solid);
        let mut iter = points.iter();
        let start = iter.next().unwrap();
        let start = Self::align_point(start, *thickness);

        if *thickness > 0. {
          canvas.move_to(start.x, start.y);
          for point in iter {
            let point = Self::align_point(point, *thickness);
            canvas.line_to(point.x, point.y);
          }
          canvas.stroke();
        }

        Self::render_endings(points, endings, canvas);
        Self::draw_caption_in(caption, used, canvas);
      }
      Shape::Dot(point, radius, caption) => {
        canvas.fill_with(*color);
        canvas.circle(point, *radius);
        let mut used = Rect::from_point_and_size(*point, (0., 0.));
        used.outset((*radius, *radius));
        Self::draw_caption_in(caption, &used, canvas);
      }
      Shape::Arrow(points, caption, endings) => {
        Self::render_line(canvas, used, points, caption, endings);
      }
      Shape::Line(points, caption, endings) =>
        Self::render_line(canvas, used, points, caption, endings),
      Shape::Text(paragraph, _) => {
        if paragraph.widths.len() > 1 {
          Self::render_paragraph(canvas, used, &paragraph.text);
        } else {
          canvas.text(&paragraph.text, (used.left, used.top + canvas.font.metrics().0));
        }
      }
      _ => warn!("Cannot render: {:?}", shape),
    }
  }

  fn render_line(canvas: &mut Canvas, used: &Rect, points: &[Point], caption: &Option<Caption>, endings: &Endings) {
    canvas.paint.set_style(PaintStyle::Stroke);
    let mut iter = points.iter();
    let start = Self::align_point(iter.next().unwrap(), 1.);
    canvas.move_to(start.x, start.y);

    for point in iter {
      let point = Self::align_point(point, 1.);
      canvas.line_to(point.x, point.y);
    }

    canvas.stroke();

    Self::render_endings(points, endings, canvas);
    Self::draw_caption_in(caption, used, canvas);
  }

  fn render_endings(points: &[Point], endings: &Endings, canvas: &mut Canvas) {
    let mut iter = points.iter().rev();
    let last = iter.next().unwrap();
    let prev = iter.next().unwrap();
    Self::draw_ending(&endings.end, last, prev, canvas);
    let mut iter = points.iter();
    let first = iter.next().unwrap();
    let next = iter.next().unwrap();
    Self::draw_ending(&endings.start, first, next, canvas);
  }

  fn draw_ending(ending: &Ending, last: &Point, before: &Point, canvas: &mut Canvas) {
    match ending {
      Ending::Dot => Self::draw_dot(canvas, last),
      Ending::Arrow => {
        let direction = last.sub(*before);
        Self::draw_arrow_head(canvas, last, direction);
      }
      _ => {}
    }
  }

  fn align_point(point: &Point, thickness: f32) -> Point {
    let mut aligned = Point::new(point.x.trunc(), point.y.trunc());
    if thickness.trunc() % 2. != 0. {
      aligned.offset((0.5, 0.5));
    }
    aligned
  }

  fn align_rect(rect: &Rect, thickness: f32) -> Rect {
    let aligned = Self::align_point(&(rect.left, rect.top).into(), thickness);
    Rect::from_point_and_size(aligned, (rect.width().round(), rect.height().round()))
  }

  pub fn dot_offset_of(point: &Point, radius: &Radius, caption: &Caption) -> Rect {
    let mut used = Rect::from_point_and_size(*point, (0., 0.));
    used.outset((radius * 2., radius * 2.));
    caption.place_in_rect(&used)
  }

  fn draw_caption_in(caption: &Option<Caption>, used: &Rect, canvas: &mut Canvas) {
    if let Some(caption) = caption {
      let rect = caption.place_in_rect(used);
      let mut topleft = Point::new(rect.left, rect.bottom);

      if caption.opaque {
        let mut rect = Self::align_rect(&rect, 1.);
        rect.outset((TEXT_PADDING, TEXT_PADDING));
        let color = canvas.paint.color();
        canvas.paint.set_color(Color::LIGHT_GRAY);
        canvas.paint.set_style(PaintStyle::StrokeAndFill);
        canvas.rectangle(&rect, 0.);
        canvas.paint.set_color(color);
      }

      topleft.offset((0., -caption.bounds.bottom));
      canvas.paint.set_style(PaintStyle::Fill);
      canvas.text(&caption.text, topleft);
    }
  }

  fn paint_paragraph(canvas: &mut Canvas, used: &Rect, text_color: &Color, paragraph: &Option<Paragraph>) {
    if let Some(paragraph) = paragraph {
      canvas.paint.set_color(*text_color);
      canvas.paint.set_style(PaintStyle::Fill);
      let paragraph = canvas.paragraph(&paragraph.text, used.width() - TEXT_PADDING * 2., TextAlign::Center);

      let mut top_left = Point::from((used.left, used.top));
      top_left.offset((TEXT_PADDING, (used.height() - paragraph.height()) / 2.));
      let top_left = Self::align_point(&top_left, 1.);
      canvas.paint_paragraph(&paragraph, top_left);
    }
  }

  #[allow(dead_code)]
  fn draw_paragraph(canvas: &mut Canvas, used: &Rect, text_color: &Color, paragraph: &Option<Paragraph>) {
    if let Some(paragraph) = paragraph {
      canvas.paint.set_style(PaintStyle::Fill);
      canvas.paint.set_color(*text_color);
      let mut rect = *used;
      if paragraph.widths.len() == 1 {
        rect.top += (used.height() - paragraph.height) / 2. - canvas.get_font_descent();
        rect.left += (used.width() - paragraph.widths.first().unwrap()) / 2.;
      } else {
        rect = rect.with_inset((TEXT_PADDING, TEXT_PADDING));
      }
      let rect = Self::align_rect(&rect, 1.);
      canvas.draw_paragraph(&paragraph.text, (rect.left, rect.top), rect.width());
    }
  }

  fn draw_arrow_head(canvas: &mut Canvas, p2: &Point, direction: Point) {
    let angle = direction.y.atan2(direction.x);
    let arrow = 25. * PI / 180.;
    let size = 15.;
    canvas.move_to(p2.x - size * (angle - arrow).cos(), p2.y - size * (angle - arrow).sin());
    canvas.line_to(p2.x, p2.y);
    canvas.line_to(p2.x - size * (angle + arrow).cos(), p2.y - size * (angle + arrow).sin());
    canvas.fill();
  }

  fn render_paragraph(canvas: &mut Canvas, rect: &Rect, title: &str) {
    let origin = (rect.left, rect.top);
    canvas.draw_paragraph(title, origin, rect.width());
  }

  #[allow(dead_code)]
  fn final_placement(nodes: &mut [Node]) {
    for node in nodes.iter_mut() {
      match node {
        Container(_attrs, used, nodes) => {
          used.top += 16.;
          Self::final_placement(nodes);
        }
        Primitive(common, _) => {
          common.used.top += 16.;
        }
        _ => {}
      }
    }
  }
  fn draw_dot(canvas: &mut Canvas, point: &Point) {
    canvas.paint.set_style(PaintStyle::Fill);
    canvas.paint.set_color(Color::BLACK);
    canvas.circle(point, 4.);
  }
}

#[cfg(test)]
mod tests {
  use skia_safe::Rect;

  use crate::diagram::renderer::Renderer;

  #[test]
  fn align_rect() {
    let aligned = Renderer::align_rect(&Rect::from_xywh(0., 0., 1., 1.), 1.);
    assert_eq!(aligned, Rect::from_xywh(0.5, 0.5, 1., 1.));

    let aligned = Renderer::align_rect(&Rect::from_xywh(0.5, 0.5, 1., 1.), 1.);
    assert_eq!(aligned, Rect::from_xywh(0.5, 0.5, 1., 1.));
  }

  #[test]
  fn align_point() {
    let aligned = Renderer::align_point(&(0, 0).into(), 1.);
    assert_eq!(aligned, (0.5, 0.5).into());

    let aligned = Renderer::align_point(&(0.5, 0.5).into(), 1.);
    assert_eq!(aligned, (0.5, 0.5).into());

    let aligned = Renderer::align_point(&(0, 0).into(), 2.);
    assert_eq!(aligned, (0, 0).into());

    let aligned = Renderer::align_point(&(0.5, 0.5).into(), 2.);
    assert_eq!(aligned, (0, 0).into());

    let aligned = Renderer::align_point(&(0, 0).into(), 3.);
    assert_eq!(aligned, (0.5, 0.5).into());

    let aligned = Renderer::align_point(&(0.5, 0.5).into(), 3.);
    assert_eq!(aligned, (0.5, 0.5).into());

    let aligned = Renderer::align_point(&(0, 0).into(), 4.);
    assert_eq!(aligned, (0, 0).into());
  }
}
