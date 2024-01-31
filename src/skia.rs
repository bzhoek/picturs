use std::fs::File;
use std::io::Write;
use std::mem;

use skia_safe::{Color, Data, EncodedImageFormat, Font, ISize, Paint, PaintStyle, Path, PathEffect, Point, Rect, scalar, Surface, surfaces, Typeface};

pub struct Canvas {
  pub surface: Surface,
  path: Path,
  pub paint: Paint,
}

impl Canvas {
  pub fn new(size: impl Into<ISize>) -> Canvas {
    let mut surface = surfaces::raster_n32_premul(size).expect("surface");
    let path = Path::new();
    let mut paint = Paint::default();
    paint.set_color(Color::BLACK);
    paint.set_anti_alias(true);
    paint.set_stroke_width(1.0);
    surface.canvas().clear(Color::LIGHT_GRAY);

    Canvas {
      surface,
      path,
      paint,
    }
  }

  #[inline]
  pub fn save(&mut self) {
    self.canvas().save();
  }

  pub fn path_effect(&mut self) {
    self.paint.set_path_effect(PathEffect::discrete(10.0, 0.5, None));
  }

  pub fn translate(&mut self, dx: f32, dy: f32) {
    self.canvas().translate((dx, dy));
  }

  pub fn scale(&mut self, sx: f32, sy: f32) {
    self.canvas().scale((sx, sy));
  }

  pub fn move_to(&mut self, x: f32, y: f32) {
    self.begin_path();
    self.path.move_to((x, y));
  }

  pub fn line_to(&mut self, x: f32, y: f32) {
    self.path.line_to((x, y));
  }

  pub fn quad_to(&mut self, cpx: f32, cpy: f32, x: f32, y: f32) {
    self.path.quad_to((cpx, cpy), (x, y));
  }

  #[allow(dead_code)]
  pub fn cubic_to(&mut self, cp1x: f32, cp1y: f32, cp2x: f32, cp2y: f32, x: f32, y: f32) {
    self.path.cubic_to((cp1x, cp1y), (cp2x, cp2y), (x, y));
  }

  #[allow(dead_code)]
  pub fn close_path(&mut self) {
    self.path.close();
  }

  pub fn begin_path(&mut self) {
    let new_path = Path::new();
    self.surface.canvas().draw_path(&self.path, &self.paint);
    let _ = mem::replace(&mut self.path, new_path);
  }

  pub fn stroke(&mut self) {
    self.paint.set_style(PaintStyle::Stroke);
    self.surface.canvas().draw_path(&self.path, &self.paint);
  }

  pub fn fill(&mut self) {
    self.paint.set_style(PaintStyle::Fill);
    self.surface.canvas().draw_path(&self.path, &self.paint);
  }

  pub fn set_line_width(&mut self, width: f32) {
    self.paint.set_stroke_width(width);
  }

  pub fn text(&mut self, text: &str, origin: impl Into<Point>) {
    let font = Font::from_typeface_with_params(Typeface::default(), 17.0, 1.0, 0.0);
    self.surface.canvas().draw_str(text, origin, &font, &self.paint);
  }

  pub fn paragraph(&mut self, text: &str, origin: impl Into<Point>, width: f32) -> (Vec<scalar>, scalar) {
    let font = Font::from_typeface_with_params(Typeface::default(), 17.0, 1.0, 0.0);
    let (font_height, _font_metrics) = font.metrics();
    let advance = font_height / 4.;

    let origin = origin.into();
    let (mut x, mut y) = (0.0, font_height);
    let mut widths: Vec<scalar> = vec!();

    for word in text.split_whitespace() {
      let (word_width, _word_rect) = font.measure_str(word, None);
      if x + word_width > width {
        y += font_height;
        widths.push(x.ceil());
        x = 0.;
      }
      self.surface.canvas().draw_str(word, (origin.x + x, origin.y + y), &font, &self.paint);
      x += word_width + advance;
    }
    widths.push(x.ceil());
    (widths, y)
  }

  pub fn rectangle(&mut self, rect: &Rect, radius: f32) {
    self.surface.canvas().draw_round_rect(rect, radius, radius, &self.paint);
  }

  pub fn circle(&mut self, point: &Point, radius: f32) {
    self.surface.canvas().draw_circle(*point, radius, &self.paint);
  }

  pub fn ellipse(&mut self, rect: &Rect) {
    self.surface.canvas().draw_oval(rect, &self.paint);
  }

  pub fn data(&mut self) -> Data {
    let image = self.surface.image_snapshot();
    let mut context = self.surface.direct_context();
    image
      .encode(context.as_mut(), EncodedImageFormat::PNG, None)
      .unwrap()
  }

  pub fn write_png(&mut self, path: &str) {
    let mut file = File::create(path).unwrap();
    let data = self.data();
    let bytes = data.as_bytes();
    file.write_all(bytes).unwrap();
  }

  fn canvas(&mut self) -> &skia_safe::Canvas {
    self.surface.canvas()
  }

  pub fn get_font_descent() -> scalar {
    let font = Font::from_typeface_with_params(Typeface::default(), 17.0, 1.0, 0.0);
    let (_font_height, font_metrics) = font.metrics();
    font_metrics.descent
  }
}

