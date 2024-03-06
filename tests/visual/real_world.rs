#[cfg(test)]
mod real_world {
  use picturs::assert_diagram;
  use picturs::diagram::create_diagram;

  #[test]
  fn pic8259_test() {
    let string = r#"
      box.pic1 ht 2in wd 1in "Primary Interrupt Controller"
      line from 1/9 pic1.w 1.5in left "Timer" ljust opaque ->
      box.pic2 same "Secondary Interrupt Controller" 2.5in left from pic1
      "#;
    let diagram = create_diagram(string);
    assert_diagram!(diagram);
  }

  #[test]
  fn pic8259_diagram() {
    let string = r#"
      box.pic1 ht 2in wd 1in "Primary Interrupt Controller"
      line from 3/9 pic1.w 2in left <->
      line from 1/9 pic1.w 1.5in left "Timer" ljust opaque ->
      line from 2/9 pic1.w "Keyboard" same
      line from 4/9 pic1.w "Serial Port 2" same
      line from 5/9 pic1.w "Serial Port 1" same
      line from 6/9 pic1.w "Parallel Port 2/3" same
      line from 7/9 pic1.w "Floppy Disk" same
      line from 8/9 pic1.w "Parallel Port 1" same
      box.pic2 same "Secondary Interrupt Controller" 2.5in left from pic1
      line from 1/9 pic2.w "Real Time Clock" same
      line from 2/9 pic2.w "ACPI" same
      line from 3/9 pic2.w "Available" same
      line from 4/9 pic2.w "Available" same
      line from 5/9 pic2.w "Mouse" same
      line from 6/9 pic2.w "Co-Processor" same
      line from 7/9 pic2.w "Primary ATA" same
      line from 8/9 pic2.w "Secondary ATA" same
      "#;
    let diagram = create_diagram(string);
    assert_diagram!(diagram);
  }

  #[test]
  fn dependencies_serial() {
    let string = r#"
      right
      box.ui13 ht 1in wd 2in "jQuery"
      box.ui20 same wd 4in "Vue"
      box.ui23 same wd 1in "Revert"
      box.ui24 same wd 0.5in "UX"
      "#;
    let diagram = create_diagram(string);
    assert_diagram!(diagram);
  }

  #[test]
  fn dependencies_parallel() {
    let string = r#"
      top
      path.ui13 1in up 3in right 0.5in down 2in left 0.5in down 1in left "jQuery 😇" above
      path.ui20 1in right 1in down 3in left 0.5in up 1in right "Vue" below
      path.fix 1in right to ui20.se 1in up "Improve" above
      path.ux 1in down 1in left to fix.ne "UX" below
      line from 1/3 ui20.e to fix.ne
      line from ui20.se to 2/3 fix.e
      "#;
    let diagram = create_diagram(string);
    assert_diagram!(diagram);
  }

  #[test]
  fn timeline() {
    let string = r#"
      sline.time 4in right ->
      dot at 1/10 time.n "1949" below
      sline from 1/4 time.s 1in up "Start" *-
      "#;
    let diagram = create_diagram(string);
    assert_diagram!(diagram);
  }
}