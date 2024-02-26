use std::fs;
use std::path::Path;
use std::process::Command;

use crate::diagram::parser::Diagram;
use crate::skia::Canvas;

#[macro_export]
macro_rules! assert_canvas {
  ($canvas:expr) => {
    fn stub() {}
    fn type_name_of<T>(_: T) -> &'static str {
        std::any::type_name::<T>()
    }

    let function_name = type_name_of(stub).rsplit("::").collect::<Vec<_>>();
      picturs::test::assert_canvas_from_function($canvas, function_name);
  };
}

#[macro_export]
macro_rules! assert_diagram {
  ($diagram:expr) => {
    fn stub() {}
    fn type_name_of<T>(_: T) -> &'static str {
        std::any::type_name::<T>()
    }

    let function_name = type_name_of(stub).rsplit("::").collect::<Vec<_>>();
      picturs::test::assert_diagram_from_function($diagram, function_name);
  };
}

pub fn assert_diagram_from_function(diagram: Diagram, function_name: Vec<&str>) {
  let prefix = derive_prefix(function_name);
  assert_diagram(diagram, &prefix).unwrap();
}

pub fn assert_canvas_from_function(canvas: Canvas, function_name: Vec<&str>) {
  let prefix = derive_prefix(function_name);
  assert_canvas(canvas, &prefix).unwrap();
}

fn derive_prefix(function_name: Vec<&str>) -> String {
  let path = Path::new(file!());
  let parent = Path::new("tests/snapshots");
  if !parent.exists() {
    fs::create_dir(parent).unwrap_or_else(|_| panic!("Cannot create {:?}.", parent));
  }

  let stem = parent.join(path.file_stem().unwrap());

  let prefix = format!("{}-{}", stem.to_str().unwrap(), function_name[1]);
  prefix
}


fn assert_canvas(mut canvas: Canvas, prefix: &str) -> anyhow::Result<()> {
  let last_file = format!("{}-last.png", prefix);
  canvas.write_png(&last_file);
  assert_png(prefix, &last_file)
}

fn assert_diagram(mut diagram: Diagram, prefix: &str) -> anyhow::Result<()> {
  let last_file = format!("{}-last.png", prefix);
  diagram.shrink_to_file(&last_file);
  assert_png(prefix, &last_file)
}

pub fn assert_png(prefix: &str, last_file: &str) -> anyhow::Result<()> {
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
      .arg(ref_file)
      .arg(&diff_file)
      .output()?;
    assert!(output.status.success());
    fs::remove_file(last_file)?;
    fs::remove_file(diff_file)?;
  }
  Ok(())
}