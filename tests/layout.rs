// from https://github.com/rust-skia/rust-skia/blob/master/skia-org/src/skparagraph_example.rs

#[cfg(test)]
mod tests {
  use std::fs;
  use skia_safe::{Canvas, FontMgr, Paint, Point};
  use skia_safe::textlayout::{FontCollection, Paragraph, ParagraphBuilder, ParagraphStyle, TextStyle, TypefaceFontProvider};
  use picturs::assert_canvas;

  #[test]
  fn textlayout() {
    let mut paragraph = draw_text(LOREM_IPSUM);
    paragraph.layout(256.0);
    assert_eq!(paragraph.height(), 255.);

    let mut paragraph = draw_text(EMOJI_IPSUM);
    paragraph.layout(256.0);
    assert_eq!(paragraph.height(), 27.);

    let mut canvas = picturs::skia::Canvas::new((420, 420));
    paragraph.paint(canvas.surface.canvas(), (16, 16));
    assert_canvas!(canvas);
  }

  fn draw_text(text: &str) -> Paragraph {
    let mut font_collection = FontCollection::new();
    font_collection.set_default_font_manager(FontMgr::new(), None);

    let paragraph_style = ParagraphStyle::new();
    let mut paragraph_builder = ParagraphBuilder::new(&paragraph_style, font_collection);
    let mut ts = TextStyle::new();
    ts.set_foreground_paint(&Paint::default());
    ts.set_font_size(17.);
    paragraph_builder.push_style(&ts);
    paragraph_builder.add_text(text);
    paragraph_builder.build()
  }

  #[allow(dead_code)]
  fn draw_lorem_ipsum_ubuntu_font(canvas: &Canvas) {
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

  static EMOJI_IPSUM: &str = "明るい ipsum dolor sit 😇";
  static LOREM_IPSUM: &str = "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Curabitur at leo at nulla tincidunt placerat. Proin eget purus augue. Quisque et est ullamcorper, pellentesque felis nec, pulvinar massa. Aliquam imperdiet, nulla ut dictum euismod, purus dui pulvinar risus, eu suscipit elit neque ac est. Nullam eleifend justo quis placerat ultricies. Vestibulum ut elementum velit. Praesent et dolor sit amet purus bibendum mattis. Aliquam erat volutpat.";
}

