use std::f32::consts::PI;
use std::ops::{Add, Sub};
use log::warn;

use skia_safe::{Color, PaintStyle, Point, Rect};

use crate::diagram::index::Index;
use crate::diagram::parser::TEXT_PADDING;
use crate::diagram::types::{Node, Paragraph, Shape};
use crate::diagram::types::Node::{Container, Primitive};
use crate::skia::Canvas;

pub struct Renderer {}

impl Renderer {
  pub fn render_to_canvas(canvas: &mut Canvas, nodes: &[Node]) {
    for node in nodes.iter() {
      match node {
        Container(_id, radius, title, _rect, used, nodes) => {
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
          canvas.rectangle(used, radius.pixels());
        }
        Primitive(_id, _, used, color, shape) => {
          Self::render_shape(canvas, used, color, shape);
        }
      }
    }
  }
  fn render_shape(canvas: &mut Canvas, used: &Rect, color: &Color, shape: &Shape) {
    match shape {
      Shape::Dot(edge, radius) => {
        let point = Index::point_from_rect(used, &edge.edge, &[]);
        canvas.paint.set_style(PaintStyle::Fill);
        canvas.paint.set_color(*color);
        canvas.circle(&point, radius.pixels());
      }
      Shape::Arrow(_, from, distance, to) => {
        canvas.move_to(used.left, used.top);
        let mut point = Point::new(used.left, used.top);
        if let Some(distance) = distance {
          point = point.add(distance.offset());

          if distance.is_horizontal() {
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

          let direction = p2.sub(p1);
          Self::draw_arrow_head(canvas, p2, direction);
        }
      }
      Shape::Line(_, _, displacement, _) => {
        canvas.move_to(used.left, used.top);
        let mut point = Point::new(used.left, used.top);
        if let Some(displacement) = displacement {
          point = point.add(displacement.offset());

          if displacement.is_horizontal() {
            canvas.line_to(point.x, point.y);
            canvas.line_to(point.x, used.bottom);
          } else {
            canvas.line_to(point.x, point.y);
            canvas.line_to(used.right, point.y);
          }
        }

        canvas.line_to(used.right, used.bottom);
        canvas.stroke();
      }
      Shape::Rectangle(text_color, paragraph, radius, fill, _) => {
        canvas.paint.set_style(PaintStyle::Stroke);
        canvas.paint.set_color(*color);
        canvas.rectangle(used, radius.pixels());

        canvas.paint.set_style(PaintStyle::Fill);
        canvas.paint.set_color(*fill);
        canvas.rectangle(used, radius.pixels());

        Self::draw_paragraph(canvas, used, text_color, paragraph);
      }
      Shape::File(text_color, paragraph, radius, fill, _) => {
        canvas.paint.set_style(PaintStyle::Stroke);
        canvas.paint.set_color(*color);
        canvas.rectangle(used, radius.pixels());

        canvas.paint.set_style(PaintStyle::Fill);
        canvas.paint.set_color(*fill);
        canvas.rectangle(used, radius.pixels());

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
      Shape::Text(title, _) => {
        Self::render_paragraph(canvas, used, title);
      }
      _ => warn!("unmatched shape {:?}", shape)
    }
  }

  fn draw_paragraph(canvas: &mut Canvas, used: &Rect, text_color: &Color, paragraph: &Option<Paragraph>) {
    if let Some(paragraph) = paragraph {
      canvas.paint.set_style(PaintStyle::Fill);
      canvas.paint.set_color(*text_color);
      let mut rect = *used;
      if paragraph.widths.len() == 1 {
        rect.top += (used.height() - paragraph.height) / 2. - Canvas::get_font_descent();
        rect.left += (used.width() - paragraph.widths.first().unwrap()) / 2.;
      } else {
        rect = rect.with_inset((TEXT_PADDING, TEXT_PADDING));
      }
      canvas.paragraph(paragraph.text, (rect.left, rect.top), rect.width());
    }
  }

  fn draw_arrow_head(canvas: &mut Canvas, p2: Point, direction: Point) {
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
        Container(_id, _, _, _rect, used, nodes) => {
          used.top += 16.;
          Self::final_placement(nodes);
        }
        Primitive(_id, _, used, _, _) => {
          used.top += 16.;
        }
      }
    }
  }
}