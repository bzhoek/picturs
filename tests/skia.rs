#[cfg(test)]
mod tests {
  use std::fs::File;
  use std::io::Write;


  use skia_safe::{Font, PaintStyle, Rect, Typeface};
  use picturs::skia::Canvas;

  #[test]
  fn font_metrics() {
    let font = Font::from_typeface_with_params(Typeface::default(), 28.0, 1.0, 0.0);
    let str = "the quick brown fox jumps over the lazy dog";
    for word in str.split_whitespace() {
      println!("{} {:?}", word, font.measure_str(word, None).0);
    }
  }

  #[test]
  fn write_png() {
    let mut canvas = Canvas::new(1024, 1024);

    canvas.set_line_width(2.0);
    let rect = Rect::from_xywh(8.0, 8.0, 1008.0, 1008.0);
    canvas.paint.set_style(PaintStyle::Stroke);
    canvas.surface.canvas().draw_rect(rect, &canvas.paint);

    canvas.scale(1.0, 1.0);
    canvas.move_to(36.0, 48.0);
    canvas.quad_to(660.0, 880.0, 1200.0, 360.0);
    canvas.translate(10.0, 10.0);
    canvas.set_line_width(4.0);
    canvas.text("Hello, world");
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
    let d = canvas.data();
    let mut file = File::create("target/test.png").unwrap();
    let bytes = d.as_bytes();
    file.write_all(bytes).unwrap();
  }
}