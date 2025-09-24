use crate::diagram::parser::Diagram;
use crate::diagram::types::Node;
use crate::diagram::types::Node::Container;
use crate::skia::Canvas;
use std::fs;
use std::path::Path;
use std::process::Command;
use skia_safe::{Color, ISize};
use crate::diagram::create_diagram;

pub fn test_canvas(size: impl Into<ISize>) -> Canvas {
  Canvas::new(size, Some(Color::LIGHT_GRAY))
}

#[macro_export]
macro_rules! assert_canvas {
  ($canvas:expr) => {
    fn stub() {}
    fn type_name_of<T>(_: T) -> &'static str {
        std::any::type_name::<T>()
    }

    let function_name = type_name_of(stub).rsplit("::").collect::<Vec<_>>();
    let path = std::path::Path::new(file!());
    picturs::test::assert_canvas_from_function($canvas, path, function_name);
  };
}

#[macro_export]
macro_rules! assert_diagram {
  ($string:expr, $color:expr) => {
    {
      fn stub() {}
      fn type_name_of<T>(_: T) -> &'static str {
          std::any::type_name::<T>()
      }

      picturs::test::assert_diagram_from_function(file!(), type_name_of(stub), $string, $color);
    };
  };
  ($string:expr) => {
    {
      fn stub() {}
      fn type_name_of<T>(_: T) -> &'static str {
          std::any::type_name::<T>()
      }

      picturs::test::assert_diagram_from_function(file!(), type_name_of(stub), $string, Some(skia_safe::Color::LIGHT_GRAY));
    };
  };
}

pub fn assert_diagram_from_function(file: &str, type_name: &str, string: &str, background: Option<Color>) {
  let path = std::path::Path::new(file);
  let function_name = type_name.rsplit("::").collect::<Vec<_>>();
  let file_prefix = derive_prefix(path, function_name);
  let diagram = create_diagram(string);
  assert_diagram(diagram, &file_prefix, background).unwrap();
}

fn derive_prefix(path: &Path, function_name: Vec<&str>) -> String {
  let parent = Path::new("tests/snapshots");
  if !parent.exists() {
    fs::create_dir(parent).unwrap_or_else(|_| panic!("Cannot create {:?}.", parent));
  }

  let stem = parent.join(path.file_stem().unwrap());

  let prefix = format!("{}-{}", stem.to_str().unwrap(), function_name[1]);
  prefix
}

pub fn assert_canvas_from_function(canvas: Canvas, path: &Path, function_name: Vec<&str>) {
  let prefix = derive_prefix(path, function_name);
  assert_canvas(canvas, &prefix).unwrap();
}

fn assert_canvas(mut canvas: Canvas, prefix: &str) -> anyhow::Result<()> {
  let last_file = format!("{}-last.png", prefix);
  canvas.write_png(&last_file);
  assert_png(prefix, &last_file, None)
}

fn assert_diagram(mut diagram: Diagram, prefix: &str, background: Option<Color>) -> anyhow::Result<()> {
  let last_file = format!("{}-last.png", prefix);
  diagram.shrink_to_file(&last_file, background);
  assert_png(prefix, &last_file, Some(&diagram))
}

pub fn assert_png(prefix: &str, last_file: &str, diagram: Option<&Diagram>) -> anyhow::Result<()> {
  let ref_file = format!("{}.png", prefix);
  let diff_file = format!("{}-diff.png", prefix);

  let compare = ["/usr/local/bin/compare", "/opt/homebrew/bin/compare"].iter().find(|path| {
    Path::new(path).exists()
  }).unwrap_or_else(|| panic!("compare not found"));

  if !Path::new(&ref_file).exists() {
    fs::rename(last_file, ref_file)?;
    if Path::new(&diff_file).exists() {
      fs::remove_file(diff_file)?;
    }
  } else {
    let output = Command::new(compare)
      .arg("-metric")
      .arg("rmse")
      .arg(last_file)
      .arg(&ref_file)
      .arg(&diff_file)
      .output()?;
    if !output.status.success() {
      if let Some(diagram) = diagram {
        dump_nested(1, &diagram.nodes);
      }
      panic!("difference {} between {} and {}", String::from_utf8(output.stderr)?, last_file, ref_file);
    }
    fs::remove_file(last_file)?;
    fs::remove_file(diff_file)?;
  }
  Ok(())
}

pub fn dump_nested(level: usize, nodes: &[Node]) {
  let indent = "  ".repeat(level - 1);
  for (index, node) in nodes.iter().enumerate() {
    match node {
      Node::Grid => {
        println!("{} {}. Grid", indent, index);
      }
      Container(attrs, used, nodes) => {
        println!("{} {}. Container used: {:?} attrs {:?}", indent, index, used, attrs);
        dump_nested(level + 1, nodes);
      }
      Node::Closed(attrs, used, _, shape) => {
        println!("{} {}. Closed {:?} used {:?} attrs: {:?}", indent, index, shape, used, attrs);
      }
      Node::Open(attrs, used, shape) => {
        println!("{} {}. Open {:?} used {:?} attrs: {:?}", indent, index, shape, used, attrs);
      }
      Node::Primitive(attrs, shape) => {
        println!("{} {}. Primitive {:?} {:?}", indent, index, shape, attrs);
      }
      Node::Font(_) => {}
      Node::Move(rect) => {
        println!("{} {}. Move used {:?}", indent, index, rect);
      }
    }
  }
}
