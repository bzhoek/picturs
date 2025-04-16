// from https://github.com/rust-skia/rust-skia/blob/master/skia-org/src/skparagraph_example.rs

#[cfg(test)]
mod tests {
  use std::fs;

  use skia_safe::textlayout::{FontCollection, ParagraphBuilder, ParagraphStyle, TextAlign, TextStyle, TypefaceFontProvider};
  use skia_safe::PaintStyle::Stroke;
  use skia_safe::{FontMgr, Paint, Point, Rect, Size};

  use picturs::assert_canvas;
  use picturs::test::test_canvas;

  #[test]
  fn layout_lorem() {
    let mut canvas = test_canvas((420, 420));

    let paragraph = canvas.paragraph(LOREM_IPSUM, 256.0, TextAlign::Left);
    canvas.paint_paragraph(&paragraph, (16, 16));
    assert_canvas!(canvas);
  }

  #[test]
  fn size() {
    let mut canvas = test_canvas((120, 80));

    let mut paragraph = canvas.paragraph("LOREMSES\nIPSUM", 256.0, TextAlign::Center);
    let size = Size::new(paragraph.min_intrinsic_width(), paragraph.height());
    paragraph.layout(size.width.ceil());
    assert_eq!(size.width, 94.47);
    assert_eq!(size.height, 34.);

    let origin = Point::new(8., 8.);
    canvas.paint_paragraph(&paragraph, origin);

    canvas.paint.set_style(Stroke);
    let rect = Rect::from_point_and_size(origin, size);
    canvas.rectangle(&rect, 0.);
    assert_canvas!(canvas);
  }

  #[test]
  fn layout_japanese() {
    let mut canvas = test_canvas((420, 420));

    let paragraph = canvas.paragraph("LOREM_IPSUM", 256.0, TextAlign::Left);
    assert_eq!(paragraph.min_intrinsic_width(), 123.75);
    assert_eq!(paragraph.max_intrinsic_width(), 123.75);

    let paragraph = canvas.paragraph(LOREM_IPSUM, 256.0, TextAlign::Left);
    assert_eq!(paragraph.min_intrinsic_width(), 96.41);
    assert_eq!(paragraph.max_intrinsic_width(), 3276.11); // apparently the whole line
    assert_eq!(paragraph.height(), 255.);

    let paragraph = canvas.paragraph(EMOJI_IPSUM, 320.0, TextAlign::Left);
    assert_eq!(paragraph.height(), 79.);

    canvas.paint_paragraph(&paragraph, (16, 16));
    assert_canvas!(canvas);
  }

  #[allow(dead_code)]
  fn draw_lorem_ipsum_ubuntu_font(canvas: &skia_safe::Canvas) {
    let ubuntu_regular: &[u8] = &fs::read("Ubuntu-Regular.ttf").unwrap();
    const TYPEFACE_ALIAS: &str = "ubuntu-regular";

    let typeface_font_provider = {
      let mut typeface_font_provider = TypefaceFontProvider::new();
      // We need a system font manager to be able to load typefaces.
      let font_mgr = FontMgr::new();
      let typeface = font_mgr
        .new_from_data(ubuntu_regular, None)
        .expect("Failed to load Ubuntu font");

      typeface_font_provider.register_typeface(typeface, TYPEFACE_ALIAS.into());
      typeface_font_provider
    };

    let mut font_collection = FontCollection::new();
    font_collection.set_default_font_manager(Some(typeface_font_provider.into()), None);
    let paragraph_style = ParagraphStyle::new();
    let mut paragraph_builder = ParagraphBuilder::new(&paragraph_style, font_collection);
    let mut ts = TextStyle::new();
    ts.set_foreground_paint(&Paint::default())
      .set_font_families(&[TYPEFACE_ALIAS]);
    paragraph_builder.push_style(&ts);
    paragraph_builder.add_text(LOREM_IPSUM);
    let mut paragraph = paragraph_builder.build();
    paragraph.layout(256.0);
    paragraph.paint(canvas, Point::default());
  }

  static EMOJI_IPSUM: &str = "明るい ソフトウェア製品生産管理:\nソフトウェア工学における 品質管理(QC)と品質保証 (QA) 発表 😇";
  static LOREM_IPSUM: &str = "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Curabitur at leo at nulla tincidunt placerat. Proin eget purus augue. Quisque et est ullamcorper, pellentesque felis nec, pulvinar massa. Aliquam imperdiet, nulla ut dictum euismod, purus dui pulvinar risus, eu suscipit elit neque ac est. Nullam eleifend justo quis placerat ultricies. Vestibulum ut elementum velit. Praesent et dolor sit amet purus bibendum mattis. Aliquam erat volutpat.";
}

