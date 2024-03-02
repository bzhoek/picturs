#[cfg(test)]
mod tests {
  use skia_safe::{Font, FontMgr, FontStyle, PaintStyle, Point, Rect, Size};

  use picturs::assert_canvas;
  use picturs::diagram::types::{Config, Edge};
  use picturs::skia::Canvas;

  static TQBF: &str = "the quick brown fox jumps over the lazy dog";

  #[test]
  fn typeface() {
    let typeface = FontMgr::default().match_family_style("Helvetica", FontStyle::default()).unwrap();
    let font = Font::from_typeface(typeface, 17.0);
    let (height, _metrics) = font.metrics();
    assert_eq!(height, 17.)
  }

  #[test]
  fn measure_str() {
    let config = Config::default();
    let bounds = config.measure_string(TQBF);
    assert_eq!(bounds, Size::new(331., 17.));
  }

  #[test]
  fn measure_whitespace_str() {
    let config = Config::default();
    let bounds = config.measure_string(" TQBF ");
    assert_eq!(bounds, Size::new(55., 17.));
  }

  #[test]
  fn center_str() {
    let config = Config::default();
    let rect = Rect::from_xywh(40., 40., 10., 20.);
    let bounds = config.measure_string("Title");
    let edge = Edge::from("c");
    let bounds = Rect::from_size(bounds);
    let offset = edge.topleft_offset(&bounds);
    let center = rect.center();
    let topleft = center + offset;

    assert_eq!(topleft, Point::from((28.5, 41.5)));
  }

  #[test]
  fn draw_paragraph() {
    let mut canvas = Canvas::new((1024, 1024));

    assert!(canvas.paint.is_anti_alias());
    canvas.paint.set_style(PaintStyle::Stroke);
    // canvas.paint.set_stroke_width(0.);
    canvas.rectangle(&Rect::from_xywh(0.5, 0.5, 320., 240.), 0.);

    canvas.paint.set_style(PaintStyle::Fill);
    let (widths, height) = canvas.paragraph(TQBF, (40, 40), 320.);
    assert_canvas!(canvas);

    assert_eq!(widths, vec!(299.0, 33.0));
    assert_eq!(height, 34.);
  }

  #[test]
  fn draw_demo() {
    let mut canvas = Canvas::new((1024, 1024));

    canvas.set_line_width(2.0);
    let rect = Rect::from_xywh(8.0, 8.0, 1008.0, 1008.0);
    canvas.paint.set_style(PaintStyle::Stroke);
    canvas.surface.canvas().draw_rect(rect, &canvas.paint);

    canvas.scale(1.0, 1.0);
    canvas.move_to(36.0, 48.0);
    canvas.quad_to(660.0, 880.0, 1200.0, 360.0);
    canvas.translate(10.0, 10.0);
    canvas.set_line_width(4.0);
    canvas.text("Hello, world", (32., 320.));
    canvas.stroke();
    canvas.save();

    let rect1 = Rect::from_xywh(16.0, 16.0, 400.0, 400.0);
    let rect2 = Rect::from_xywh(416.0, 16.0, 400.0, 400.0);
    canvas.paint.set_style(PaintStyle::Stroke);
    canvas.surface.canvas().draw_rect(rect1, &canvas.paint);
    canvas.surface.canvas().draw_rect(rect2, &canvas.paint);

    canvas.move_to(530.0, 90.0);
    canvas.line_to(610.0, 20.0);
    canvas.line_to(740.0, 130.0);
    canvas.line_to(560.0, 130.0);
    canvas.line_to(690.0, 20.0);
    canvas.line_to(770.0, 90.0);
    canvas.fill();
    assert_canvas!(canvas);
  }

  #[test]
  fn draw_quad() {
    let mut canvas = Canvas::new((1024, 1024));

    canvas.move_to(40.0, 40.0);
    canvas.quad_to(440.0, 440.0, 840.0, 40.0);
    canvas.stroke();
    canvas.save();

    assert_canvas!(canvas);
  }

  #[test]
  fn draw_cubic() {
    let mut canvas = Canvas::new((1024, 1024));

    canvas.move_to(36.0, 48.0);
    canvas.cubic_to(36.0, 400.0, 1000.0, 400.0, 1000.0, 48.0);
    canvas.stroke();
    canvas.save();

    assert_canvas!(canvas);
  }
}