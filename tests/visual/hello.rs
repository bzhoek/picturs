#[cfg(test)]
mod hello {
  use picturs::test::assert_diagram;

  use crate::common::create_diagram;

  #[test]
  fn visual_hello_statements() -> anyhow::Result<()> {
    let string = r#"
      set box pd 0
      right
      box "box"
      circle "circle" 2cm right
      ellipse "ellipse" 2cm right
      oval "oval" 1cm down last box
      cylinder "cylinder" 1cm down last circle
      file "file" 1cm down last ellipse
      "#;
    let diagram = create_diagram(string);
    assert_diagram(diagram, "target/visual_hello_statements")?;
    Ok(())
  }

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
      set box pd 0 ht 64 wd 120
      set circle ht 64
      right
      box
      circle
      # cylinder
      "#;
    let diagram = create_diagram(string);
    assert_diagram(diagram, "target/visual_box_circle_cylinder")?;
    Ok(())
  }

  #[test]
  fn visual_box_box() -> anyhow::Result<()> {
    let string = r#"
      box { box "A" }
      box { box "B" }
      "#;
    let diagram = create_diagram(string);
    assert_diagram(diagram, "target/visual_box_box")?;
    Ok(())
  }

  #[test]
  fn visual_box_box_box() -> anyhow::Result<()> {
    let string = r#"
      box pd 0 top color green{
        box pd 0
        box pd 0
        box pd 0
      }
      box pd 0 top color blue {
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
  fn visual_direction_start_end() -> anyhow::Result<()> {
    let string = r#"
      set box pd 0 ht 32 wd 64
      box top color green {
        box "Layout Direction" wd 160
        box ".start"
        box ".end"
      }
      box top color blue {
        box "right" wd 160
        box ".w"
        box ".e"
      }
      box top color blue {
        box "down" wd 160
        box ".n"
        box ".s"
      }
      box top color blue {
        box "left" wd 160
        box ".e"
        box ".w"
      }
      box top color blue {
        box "up" wd 160
        box ".s"
        box ".n"
      }
      "#;
    let diagram = create_diagram(string);
    assert_diagram(diagram, "target/visual_direction_start_end")?;
    Ok(())
  }
}