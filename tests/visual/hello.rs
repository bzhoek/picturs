#[cfg(test)]
mod hello {
  use picturs::assert_diagram;
  use picturs::diagram::create_diagram;

  #[test]
  fn hello_statements() {
    let string = r#"
      set box pd 0
      right
      box "box"
      circle "circle" 2cm right
      ellipse "ellipse" 2cm right
      oval "oval" 1cm down last box
      file "file" 1cm down last circle
      cylinder "cylinder" 1cm down last ellipse
      "#;
    let diagram = create_diagram(string);
    assert_diagram!(diagram);
  }

  #[test]
  fn hello_world_right() {
    let string = r#"
      right
      line
      box "Hello"
      arrow
      "#;
    let diagram = create_diagram(string);
    assert_diagram!(diagram);
  }

  #[test]
  fn hello_world_down() {
    let string = r#"
      down
      line
      box "Hello"
      arrow
      "#;
    let diagram = create_diagram(string);
    assert_diagram!(diagram);
  }

  #[test]
  fn box_circle_cylinder() {
    let string = r#"
      set box pd 0 ht 64 wd 120
      set circle ht 64
      right
      box
      circle
      # cylinder
      "#;
    let diagram = create_diagram(string);
    assert_diagram!(diagram);
  }

  #[test]
  fn box_box() {
    let string = r#"
      box { box "A" }
      box { box "B" }
      "#;
    let diagram = create_diagram(string);
    assert_diagram!(diagram);
  }

  #[test]
  fn box_box_box() {
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
    assert_diagram!(diagram);
  }

  #[test]
  fn direction_start_end() {
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
    assert_diagram!(diagram);
  }
}