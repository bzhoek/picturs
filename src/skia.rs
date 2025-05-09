use std::fs::File;
use std::io::Write;
use std::mem;

use skia_safe::{Color, Data, EncodedImageFormat, Font, FontMgr, FontStyle, ISize, Paint, PaintStyle, Path, PathEffect, Point, Rect, scalar, Surface, surfaces};
use skia_safe::textlayout::{FontCollection, Paragraph, ParagraphBuilder, ParagraphStyle, StrutStyle, TextAlign, TextStyle};

pub static A5: (i32, i32) = (798, 562);

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum Effect {
  Dashed,
  Dotted,
  #[default]
  Solid,
}

pub struct Canvas {
  pub surface: Surface,
  path: Path,
  pub paint: Paint,
  pub font: Font,
  font_collection: FontCollection,
}

impl Canvas {
  pub fn new(size: impl Into<ISize>, background: Option<Color>) -> Canvas {
    let mut font_collection = FontCollection::new();
    font_collection.set_default_font_manager(FontMgr::new(), None);

    let mut surface = surfaces::raster_n32_premul(size).expect("surface");
    let path = Path::new();
    let mut paint = Paint::default();
    paint.set_color(Color::BLACK);
    paint.set_anti_alias(true);
    paint.set_stroke_width(1.0);
    if let Some(color) = background {
      surface.canvas().clear(color);
    }
    let typeface = FontMgr::default().match_family_style("Helvetica", FontStyle::default()).unwrap();
    let font = Font::from_typeface(typeface, 17.0);

    Canvas {
      surface,
      path,
      paint,
      font,
      font_collection,
    }
  }

  pub fn clear(&mut self, color: Color) {
    self.surface.canvas().clear(color);
  }

  #[inline]
  pub fn save(&mut self) {
    self.canvas().save();
  }

  pub fn path_effect(&mut self) {
    self.paint.set_path_effect(PathEffect::discrete(10.0, 0.5, None));
  }

  pub fn translate(&mut self, dx: f32, dy: f32) {
    self.canvas().translate((dx.trunc(), dy.trunc()));
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
    self.path.reset();
  }

  pub fn fill(&mut self) {
    self.paint.set_style(PaintStyle::Fill);
    self.surface.canvas().draw_path(&self.path, &self.paint);
  }

  pub fn stroke_with(&mut self, width: f32, color: Color, effect: &Effect) {
    self.paint.set_style(PaintStyle::Stroke);
    self.paint.set_stroke_width(width);
    self.paint.set_color(color);
    let effect = match effect {
      Effect::Dashed => PathEffect::dash(&[10., 10.], 0.),
      Effect::Dotted => PathEffect::dash(&[2., 4.], 0.),
      Effect::Solid => None
    };
    self.paint.set_path_effect(effect);
  }

  pub fn fill_with(&mut self, color: Color) {
    self.paint.set_style(PaintStyle::Fill);
    self.paint.set_color(color);
  }

  pub fn text(&mut self, text: &str, origin: impl Into<Point>) {
    self.surface.canvas().draw_str(text, origin, &self.font, &self.paint);
  }

  pub fn paragraph(&self, text: &str, width: f32, align: TextAlign) -> Paragraph {
    let mut textstyle = TextStyle::new();
    textstyle.set_font_size(self.font.size());
    textstyle.set_foreground_paint(&self.paint);

    let mut strutstyle = StrutStyle::new();
    strutstyle.set_font_size(self.font.size() * 1.0);
    strutstyle.set_strut_enabled(true);
    strutstyle.set_force_strut_height(true);

    let mut paragraph_style = ParagraphStyle::new();
    paragraph_style.set_text_align(align);
    // paragraph_style.set_strut_style(strutstyle);

    let mut paragraph_builder = ParagraphBuilder::new(&paragraph_style, &self.font_collection);
    paragraph_builder.push_style(&textstyle);
    paragraph_builder.add_text(text);

    let mut paragraph = paragraph_builder.build();
    paragraph.layout(width);
    paragraph
  }

  pub fn paint_paragraph(&mut self, paragraph: &Paragraph, origin: impl Into<Point>) {
    paragraph.paint(self.surface.canvas(), origin);
  }

  pub fn draw_paragraph(&mut self, text: &str, origin: impl Into<Point>, width: f32) -> (Vec<scalar>, scalar) {
    let (font_height, _font_metrics) = self.font.metrics();
    let advance = font_height / 4.;

    let origin = origin.into();
    let (mut x, mut y) = (0.0, font_height);
    let mut widths: Vec<scalar> = vec!();

    for word in text.split_whitespace() {
      let (word_width, _word_rect) = self.font.measure_str(word, None);
      if x + word_width > width {
        y += font_height;
        widths.push(x.ceil());
        x = 0.;
      }
      self.surface.canvas().draw_str(word, (origin.x + x, origin.y + y), &self.font, &self.paint);
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

  pub fn file(&mut self, rect: &Rect) {
    let fold = 16.;
    self.move_to(rect.left, rect.top);
    self.line_to(rect.left, rect.bottom);
    self.line_to(rect.right, rect.bottom);
    self.line_to(rect.right, rect.top + fold);
    self.line_to(rect.right - fold, rect.top + fold);
    self.line_to(rect.right - fold, rect.top);
    self.line_to(rect.right, rect.top + fold);
    self.move_to(rect.left, rect.top);
    self.line_to(rect.right - fold, rect.top);
    self.stroke();
  }

  pub fn cylinder(&mut self, rect: &Rect) {
    let top = Rect::from_xywh(rect.left, rect.top, rect.width(), rect.height() / 3.);
    let bottom = Rect::from_xywh(rect.left, rect.bottom - top.height(), rect.width(), top.height());
    let height = top.height() / 2.;

    self.path.arc_to(top, 180., 359., true);
    self.path.line_to((rect.left, bottom.top + height));
    self.path.arc_to(bottom, 180., -180., false);
    self.path.move_to((bottom.right, bottom.top + height));
    self.path.line_to((top.right, top.top + height));
    self.stroke();
  }

  pub fn oval(&mut self, rect: &Rect) {
    let left = Rect::from_xywh(rect.left, rect.top, rect.height(), rect.height());
    let right = Rect::from_xywh(rect.right - rect.height(), rect.top, rect.height(), rect.height());
    self.path.arc_to(left, -90., -180., true);
    self.path.line_to((right.left, right.bottom));
    self.path.arc_to(right, 90., -180., false);
    self.path.close();
    self.stroke();
  }

  pub fn data(&mut self) -> Data {
    let image = self.surface.image_snapshot();
    let mut context = self.surface.direct_context();
    image
      .encode(context.as_mut(), EncodedImageFormat::PNG, None)
      .unwrap()
  }

  pub fn write_png<P: AsRef<std::path::Path>>(&mut self, path: P) {
    let mut file = File::create(path).unwrap();
    let data = self.data();
    let bytes = data.as_bytes();
    file.write_all(bytes).unwrap();
  }

  fn canvas(&mut self) -> &skia_safe::Canvas {
    self.surface.canvas()
  }

  pub fn get_font_descent(&self) -> scalar {
    let (_font_height, font_metrics) = self.font.metrics();
    font_metrics.descent
  }
}

