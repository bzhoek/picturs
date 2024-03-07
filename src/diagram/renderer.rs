use std::f32::consts::PI;
use std::ops::{Add, Sub};

use log::warn;
use skia_safe::{Color, PaintStyle, Point, Rect};

use crate::diagram::parser::TEXT_PADDING;
use crate::diagram::types::{Caption, Displacement, Ending, Endings, Node, ObjectEdge, Paragraph, Radius, Shape};
use crate::diagram::types::Node::{Container, Primitive};
use crate::skia::Canvas;

pub struct Renderer {}

impl Renderer {
  pub fn render_to_canvas(canvas: &mut Canvas, nodes: &[Node]) {
    for node in nodes.iter() {
      canvas.paint.set_stroke_width(1.0);

      match node {
        Container(_id, radius, title, used, nodes) => {
          Self::render_to_canvas(canvas, nodes);

          if let Some(title) = title {
            canvas.paint.set_style(PaintStyle::Fill);
            canvas.paint.set_color(Color::BLACK);
            let inset = used.with_inset((TEXT_PADDING, TEXT_PADDING));
            let origin = (inset.left, inset.bottom - 16.);
            canvas.paragraph(title, origin, inset.width());
          }

          canvas.paint.set_style(PaintStyle::Stroke);
          canvas.paint.set_color(Color::RED);
          canvas.rectangle(used, *radius);
        }
        // Primitive(_id, used, color, shape) => {
        Primitive(common, shape) => {
          let used = Self::align_rect(&common.used);
          Self::render_shape(canvas, &used, &common.stroke, shape);
        }
      }
    }
  }
  fn render_shape(canvas: &mut Canvas, used: &Rect, color: &Color, shape: &Shape) {
    match shape {
      Shape::Font(font) => {
        canvas.font = font.clone();
      }
      Shape::Path(start, points, caption) => {
        canvas.paint.set_style(PaintStyle::Stroke);
        canvas.paint.set_color(*color);
        canvas.move_to(start.x, start.y);
        for point in points.iter() {
          canvas.line_to(point.x, point.y);
        }
        canvas.stroke();

        Self::draw_caption(canvas, used, caption);
      }
      Shape::Sline(thickness, points, caption, endings) => {
        canvas.paint.set_stroke_width(*thickness);
        canvas.paint.set_style(PaintStyle::Stroke);
        canvas.paint.set_color(*color);
        let mut points_iter = points.iter();
        let start = points_iter.next().unwrap();
        let start = Self::align_point(start, *thickness);

        if *thickness > 0. {
          canvas.move_to(start.x, start.y);
          for point in points_iter {
            let point = Self::align_point(point, *thickness);
            canvas.line_to(point.x, point.y);
          }
          canvas.stroke();
        }

        let end = points.last().unwrap();
        Self::draw_endings(endings, &start, end, canvas);
        Self::draw_caption(canvas, used, caption);
      }
      Shape::Dot(point, radius, caption) => {
        canvas.paint.set_style(PaintStyle::Fill);
        canvas.paint.set_color(*color);
        canvas.circle(point, *radius);
        Self::draw_dot_caption(canvas, point, radius, caption);
      }
      Shape::Arrow(from, movement, to, caption) =>
        Self::render_arrow(canvas, used, from, movement, to, caption),
      Shape::Line(start, movement, end, caption, arrows) =>
        Self::render_line(canvas, used, start, movement, end, caption, arrows),
      Shape::Box(text_color, paragraph, thickness, radius, fill, _) => {
        canvas.paint.set_style(PaintStyle::Stroke);
        canvas.paint.set_color(*color);
        canvas.paint.set_stroke_width(*thickness);

        canvas.rectangle(used, *radius);

        canvas.paint.set_style(PaintStyle::Fill);
        canvas.paint.set_color(*fill);
        canvas.rectangle(used, *radius);

        Self::draw_paragraph(canvas, used, text_color, paragraph);
      }
      Shape::File(text_color, paragraph, _radius, _fill, _) => {
        canvas.paint.set_style(PaintStyle::Stroke);
        canvas.paint.set_color(*color);

        let fold = 16.;
        canvas.move_to(used.left, used.top);
        canvas.line_to(used.left, used.bottom);
        canvas.line_to(used.right, used.bottom);
        canvas.line_to(used.right, used.top + fold);
        canvas.line_to(used.right - fold, used.top + fold);
        canvas.line_to(used.right - fold, used.top);
        canvas.line_to(used.right, used.top + fold);
        canvas.move_to(used.left, used.top);
        canvas.line_to(used.right - fold, used.top);
        canvas.stroke();

        Self::draw_paragraph(canvas, used, text_color, paragraph);
      }
      Shape::Circle(text_color, paragraph, fill, _) => {
        canvas.paint.set_style(PaintStyle::Stroke);
        canvas.paint.set_color(*color);
        canvas.circle(&used.center(), used.width() / 2.);

        canvas.paint.set_style(PaintStyle::Fill);
        canvas.paint.set_color(*fill);
        canvas.circle(&used.center(), used.width() / 2.);

        Self::draw_paragraph(canvas, used, text_color, paragraph);
      }
      Shape::Ellipse(text_color, paragraph, fill, _) => {
        canvas.paint.set_style(PaintStyle::Stroke);
        canvas.paint.set_color(*color);
        canvas.ellipse(used);

        canvas.paint.set_style(PaintStyle::Fill);
        canvas.paint.set_color(*fill);
        canvas.ellipse(used);

        Self::draw_paragraph(canvas, used, text_color, paragraph);
      }
      Shape::Cylinder(text_color, paragraph, _fill, _) => {
        canvas.paint.set_style(PaintStyle::Stroke);
        canvas.paint.set_color(*color);
        canvas.cylinder(used);

        let rect = Rect::from_xywh(used.left, used.top + used.height() / 3., used.width(), used.height() * 0.666);
        Self::draw_paragraph(canvas, &rect, text_color, paragraph);
      }
      Shape::Oval(text_color, paragraph, _fill, _) => {
        canvas.paint.set_style(PaintStyle::Stroke);
        canvas.paint.set_color(*color);
        canvas.oval(used);

        Self::draw_paragraph(canvas, used, text_color, paragraph);
      }
      Shape::Text(paragraph, _) => {
        if paragraph.widths.len() > 1 {
          Self::render_paragraph(canvas, used, &paragraph.text);
        } else {
          canvas.text(paragraph.text, (used.left, used.top + canvas.font.metrics().0));
        }
      }
      _ => warn!("unmatched shape {:?}", shape)
    }
  }

  fn render_line(canvas: &mut Canvas, used: &Rect, start: &Point, movement: &Option<Displacement>, end: &Point, caption: &Option<Caption>, endings: &Endings) {
    canvas.paint.set_style(PaintStyle::Stroke);
    canvas.move_to(used.left, used.top);
    let mut point = Point::new(used.left, used.top);
    if let Some(movement) = movement {
      point = point.add(movement.offset());

      if movement.is_horizontal() {
        canvas.line_to(point.x, point.y);
        canvas.line_to(point.x, used.bottom);
      } else {
        canvas.line_to(point.x, point.y);
        canvas.line_to(used.right, point.y);
      }
    }

    canvas.line_to(used.right, used.bottom);
    canvas.stroke();

    Self::draw_endings(endings, end, start, canvas); // FIXME the endings are reverted
    Self::draw_caption(canvas, used, caption);
  }

  fn draw_endings(endings: &Endings, start: &Point, end: &Point, canvas: &mut Canvas) {
    Self::draw_ending(&endings.start, start, end, canvas);
    Self::draw_ending(&endings.end, end, start, canvas);
  }

  fn draw_ending(ending: &Ending, at: &Point, from: &Point, canvas: &mut Canvas) {
    match ending {
      Ending::Dot => Self::draw_dot(canvas, at),
      Ending::Arrow => {
        let direction = at.sub(*from);
        Self::draw_arrow_head(canvas, at, direction);
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

  fn align_rect(used: &Rect) -> Rect {
    Rect::from_xywh(used.left.trunc() + 0.5, used.top.trunc() + 0.5, used.width().round(), used.height().round())
  }

  fn render_arrow(canvas: &mut Canvas, used: &Rect, from: &ObjectEdge, movement: &Option<Displacement>, to: &ObjectEdge, caption: &Option<Caption>) {
    canvas.move_to(used.left, used.top);
    let mut point = Point::new(used.left, used.top);
    if let Some(movement) = movement {
      point = point.add(movement.offset());

      if movement.is_horizontal() {
        canvas.line_to(point.x, point.y);
        canvas.line_to(point.x, used.bottom);
      } else {
        canvas.line_to(point.x, point.y);
        canvas.line_to(used.right, point.y);
      }
    } else {
      let p1 = if from.edge.vertical() && to.edge.horizontal() {
        Point::new(used.left, used.bottom)
      } else if from.edge.horizontal() && to.edge.vertical() {
        Point::new(used.right, used.top)
      } else {
        Point::new(used.left, used.top)
      };

      let p2 = Point::new(used.right, used.bottom);
      canvas.line_to(p1.x, p1.y);
      canvas.line_to(p2.x, p2.y);
      canvas.stroke();

      Self::draw_caption(canvas, used, caption);

      let direction = p2.sub(p1);
      Self::draw_arrow_head(canvas, &p2, direction);
    }
  }

  fn draw_dot_caption(canvas: &mut Canvas, point: &Point, radius: &Radius, caption: &Option<Caption>) {
    if let Some(caption) = caption {
      let rect = Self::dot_offset_of(point, radius, caption);
      let rect = Self::align_rect(&rect);

      canvas.paint.set_style(PaintStyle::Fill);
      canvas.text(caption.text, (rect.left, rect.bottom - (canvas.get_font_descent() / 2.)));

      // canvas.paint.set_style(PaintStyle::Stroke);
      // canvas.paint.set_color(Color::BLACK);
      // canvas.rectangle(&rect, 0.);
    }
  }

  pub fn dot_offset_of(point: &Point, radius: &Radius, caption: &Caption) -> Rect {
    let mut dot = Rect::from_point_and_size(*point, (0., 0.));
    dot.outset((radius * 2., radius * 2.));
    let point = caption.inner.mirror().edge_point(&dot);
    let mut rect = Rect::from_point_and_size(point, caption.size);
    caption.inner.offset(&mut rect);
    rect
  }

  fn draw_caption(canvas: &mut Canvas, used: &Rect, caption: &Option<Caption>) {
    if let Some(caption) = caption {
      let (topleft, rect) = Self::topleft_of(caption, used);

      if caption.opaque {
        let rect = Self::align_rect(&rect);
        canvas.paint.set_color(Color::LIGHT_GRAY);
        canvas.paint.set_style(PaintStyle::StrokeAndFill);
        canvas.rectangle(&rect, 0.);
      }

      canvas.paint.set_color(Color::BLACK);
      canvas.paint.set_style(PaintStyle::Fill);
      canvas.text(caption.text, topleft);
    }
  }

  pub fn topleft_of(caption: &Caption, used: &Rect) -> (Point, Rect) {
    let mut bounds = Rect::from_size(caption.size);
    bounds.outset((TEXT_PADDING, TEXT_PADDING));
    let offset = caption.inner.topleft_offset(&bounds);

    let mut topleft = caption.outer.edge_point(used);

    topleft.offset(offset);
    let rect = Rect::from_point_and_size(topleft, bounds.size());
    topleft.offset((TEXT_PADDING, TEXT_PADDING + bounds.height() / 2.));
    (topleft, rect)
  }

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
      let rect = Self::align_rect(&rect);
      canvas.paragraph(paragraph.text, (rect.left, rect.top), rect.width());
    }
  }

  fn draw_arrow_head(canvas: &mut Canvas, p2: &Point, direction: Point) {
    let angle = direction.y.atan2(direction.x);
    let arrow = 25. * PI / 180.;
    let size = 15.;
    canvas.move_to(
      p2.x - size * (angle - arrow).cos(),
      p2.y - size * (angle - arrow).sin());
    canvas.line_to(p2.x, p2.y);
    canvas.line_to(
      p2.x - size * (angle + arrow).cos(),
      p2.y - size * (angle + arrow).sin());
    canvas.fill();
  }

  fn render_paragraph(canvas: &mut Canvas, rect: &Rect, title: &&str) {
    let origin = (rect.left, rect.top);
    canvas.paragraph(title, origin, rect.width());
  }

  #[allow(dead_code)]
  fn final_placement(nodes: &mut [Node]) {
    for node in nodes.iter_mut() {
      match node {
        Container(_id, _, _, used, nodes) => {
          used.top += 16.;
          Self::final_placement(nodes);
        }
        Primitive(common, _) => {
          common.used.top += 16.;
        }
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
    let aligned = Renderer::align_rect(&Rect::from_xywh(0., 0., 1., 1.));
    assert_eq!(aligned, Rect::from_xywh(0.5, 0.5, 1., 1.));

    let aligned = Renderer::align_rect(&Rect::from_xywh(0.5, 0.5, 1., 1.));
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