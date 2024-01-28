use std::fs;
use std::path::Path;
use std::process::Command;

use crate::diagram::parser::Diagram;
use crate::skia::Canvas;

pub fn assert_canvas(mut canvas: Canvas, prefix: &str) -> anyhow::Result<()> {
  let last_file = format!("{}-last.png", prefix);
  canvas.write_png(&last_file);
  assert_png(prefix, &last_file)
}

pub fn assert_diagram(diagram: Diagram, prefix: &str) -> anyhow::Result<()> {
  let last_file = format!("{}-last.png", prefix);
  diagram.render_to_file(&last_file);
  assert_png(prefix, &last_file)
}

pub fn assert_png(prefix: &str, last_file: &str) -> anyhow::Result<()> {
  let ref_file = format!("{}.png", prefix);
  let diff_file = format!("{}-diff.png", prefix);

  if !Path::new(&ref_file).exists() {
    fs::rename(last_file, ref_file)?;
    if Path::new(&diff_file).exists() {
      fs::remove_file(diff_file)?;
    }
  } else {
    let output = Command::new("/usr/local/bin/compare")
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