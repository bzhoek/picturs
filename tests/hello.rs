mod common;

#[cfg(test)]
mod tests {
  use picturs::test::assert_diagram;

  use crate::common::create_diagram;

  #[test]
  fn visual_hello_world_right() -> anyhow::Result<()> {
    let string = r#"
      right
      line
      box "Hello"
      arrow
      "#;
    let diagram = create_diagram(string);
    assert_diagram(diagram, "target/visual_hello_world_right")?;
    Ok(())
  }

  #[test]
  fn visual_hello_world_down() -> anyhow::Result<()> {
    let string = r#"
      down
      line
      box "Hello"
      arrow
      "#;
    let diagram = create_diagram(string);
    assert_diagram(diagram, "target/visual_hello_world_down")?;
    Ok(())
  }

  #[test]
  fn visual_box_circle_cylinder() -> anyhow::Result<()> {
    let string = r#"
      right
      box
      circle
      # cylinder
      "#;
    let diagram = create_diagram(string);
    dbg!(&diagram);
    assert_diagram(diagram, "target/visual_box_circle_cylinder")?;
    Ok(())
  }

  #[test]
  fn visual_box_box_box() -> anyhow::Result<()> {
    let string = r#"
      box pd 0 topright color green{
        box pd 0
        box pd 0
        box pd 0
      }
      box pd 0 topright color blue {
        box pd 0
        box pd 0
        box pd 0
      }
      "#;
    let diagram = create_diagram(string);
    assert_diagram(diagram, "target/visual_box_box_box")?;
    Ok(())
  }

  #[test]
  fn visual_config() -> anyhow::Result<()> {
    let string = r#"
      config pd 0 ht 32 wd 64
      box topright color green{
        box "Layout Direction" wd 160
        box ".start"
        box ".end"
      }
      box topright color blue {
        box "right" wd 160
        box ".w"
        box ".e"
      }
      box topright color blue {
        box "down" wd 160
        box ".n"
        box ".s"
      }
      box topright color blue {
        box "left" wd 160
        box ".e"
        box ".w"
      }
      box topright color blue {
        box "up" wd 160
        box ".s"
        box ".n"
      }
      "#;
    let diagram = create_diagram(string);
    assert_diagram(diagram, "target/visual_config")?;
    Ok(())
  }
}