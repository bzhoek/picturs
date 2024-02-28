use skia_safe::Point;
use crate::diagram::conversion::Conversion;
use crate::diagram::index::Index;
use crate::diagram::parser::{Diagram, Rule};
use crate::diagram::types::{Config, Flow};
use crate::skia::Canvas;

#[test]
fn should_copy_same_attributes_from_line() {
  let mut index = Index::default();
  let mut canvas = Canvas::new((100, 100));
  let flow = Flow::new("left");
  let config = Config::new(flow, 120., 60.);
  let cursor = Point::new(0., 0.);
  let rectangle = Conversion::pair_for(Rule::rectangle, r#"box.pic1 ht 2in wd 1in "Primary Interrupt Controller""#);
  Diagram::box_from(&rectangle, &config, &mut index, &cursor, &mut canvas);

  let line = Conversion::pair_for(Rule::rectangle, r#"box.pic1 ht 2in wd 1in "Primary Interrupt Controller""#);
  Diagram::line_from(line, &config, &mut index, &cursor, &mut canvas);

  let same = Conversion::pair_for(Rule::rectangle, r#"line from 2/8 pic1.w same "Keyboard""#);
  Diagram::line_from(same, &config, &mut index, &cursor, &mut canvas);

  dbg!(rectangle);
}
