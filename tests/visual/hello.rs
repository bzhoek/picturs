#[cfg(test)]
mod hello {
  use picturs::assert_diagram;

  #[test]
  fn hello_statements() {
    let string = r#"
      right
      box "box"
      circle "circle" 2cm right
      ellipse "ellipse" 2cm right
      oval "oval" 1cm down last box
      file "file" 1cm down last circle
      cylinder "cylinder" 1cm down last ellipse
      "#;
    assert_diagram!(string);
  }

  #[test]
  fn hello_world() {
    let string = r#"
      box "Hello" "World"
      "#;
    assert_diagram!(string);
  }

  #[test]
  fn hello_world_right() {
    let string = r#"
      right
      line
      box "Hello"
      arrow
      "#;
    assert_diagram!(string);
  }

  #[test]
  fn hello_world_down() {
    let string = r#"
      down
      line
      box "Hello"
      arrow
      "#;
    assert_diagram!(string);
  }

  #[test]
  fn box_circle_cylinder() {
    let string = r#"
      set box ht 64 wd 120
      set circle ht 64
      right
      box
      circle
      # cylinder
      "#;
    assert_diagram!(string);
  }

  #[test]
  fn box_box() {
    let string = r#"
      continue down-left
      group { box "A" }
      group { box "B" }
      "#;
    assert_diagram!(string);
  }

  #[test]
  fn box_box_box() {
    let string = r#"
      continue down-left
      group top color green {
        box
        box
        box
      }
      group top color blue {
        box
        box
        box
      }
      "#;
    assert_diagram!(string);
  }

  #[test]
  fn direction_start_end() {
    let string = r#"
      continue down-left
      set box ht 32 wd 64
      group top color green {
        box "Layout Direction" wd 160
        box ".start"
        box ".end"
      }
      group top color blue {
        box "right" wd 160
        box ".w"
        box ".e"
      }
      group top color blue {
        box "down" wd 160
        box ".n"
        box ".s"
      }
      group top color blue {
        box "left" wd 160
        box ".e"
        box ".w"
      }
      group top color blue {
        box "up" wd 160
        box ".s"
        box ".n"
      }
      "#;
    assert_diagram!(string);
  }
}