#[cfg(test)]
mod tests {
  use skia_safe::{Font, PaintStyle, Rect, Size, Typeface};

  use picturs::skia::Canvas;
  use picturs::test::assert_canvas;

  static TQBF: &str = "the quick brown fox jumps over the lazy dog";

  #[test]
  fn font_metrics() {
    let font = Font::from_typeface_with_params(Typeface::default(), 28.0, 1.0, 0.0);
    for word in TQBF.split_whitespace() {
      println!("{} {:?}", word, font.measure_str(word, None).0);
    }
  }

  #[test]
  fn measure_str() {
    let mut canvas = Canvas::new((1024, 1024));
    let bounds = canvas.measure_str(TQBF);
    assert_eq!(bounds, Size::new(333., 19.));
  }

  #[test]
  fn draw_paragraph() {
    let mut canvas = Canvas::new((1024, 1024));

    canvas.paint.set_style(PaintStyle::Stroke);
    canvas.rectangle(&Rect::from_xywh(0., 0., 320., 240.), 0.);

    canvas.paint.set_style(PaintStyle::Fill);
    let (widths, height) = canvas.paragraph(TQBF, (40, 40), 320.);
    assert_canvas(canvas, "target/draw_paragraph").unwrap();

    assert_eq!(vec!(299.0, 33.0), widths);
    assert_eq!(34., height);
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
    assert_canvas(canvas, "target/draw_demo").unwrap();
  }

  #[test]
  fn draw_quad() {
    let mut canvas = Canvas::new((1024, 1024));

    canvas.move_to(40.0, 40.0);
    canvas.quad_to(440.0, 440.0, 840.0, 40.0);
    canvas.stroke();
    canvas.save();

    assert_canvas(canvas, "target/draw_quad").unwrap();
  }

  #[test]
  fn draw_cubic() {
    let mut canvas = Canvas::new((1024, 1024));

    canvas.move_to(36.0, 48.0);
    canvas.cubic_to(36.0, 400.0, 1000.0, 400.0, 1000.0, 48.0);
    canvas.stroke();
    canvas.save();

    assert_canvas(canvas, "target/draw_cubic").unwrap();
  }
}