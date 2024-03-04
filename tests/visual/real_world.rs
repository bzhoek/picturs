#[cfg(test)]
mod real_world {
  use picturs::assert_diagram;
  use picturs::diagram::create_diagram;

  #[test]
  fn pic8259_diagram() {
    let string = r#"
      box.pic1 ht 2in wd 1in "Primary Interrupt Controller"
      line from 3/8 pic1.w 2in left <->
      line from 1/8 pic1.w 1.5in left "Timer" ljust opaque ->
      line from 2/8 pic1.w "Keyboard" same
      line from 4/8 pic1.w "Serial Port 2" same
      line from 5/8 pic1.w "Serial Port 1" same
      line from 6/8 pic1.w "Parallel Port 2/3" same
      line from 7/8 pic1.w "Floppy Disk" same
      line from 8/8 pic1.w "Parallel Port 1" same
      box.pic2 same "Secondary Interrupt Controller" 2.5in left from pic1
      line from 1/8 pic2.w "Real Time Clock" same
      line from 2/8 pic2.w "ACPI" same
      line from 3/8 pic2.w "Available" same
      line from 4/8 pic2.w "Available" same
      line from 5/8 pic2.w "Mouse" same
      line from 6/8 pic2.w "Co-Processor" same
      line from 7/8 pic2.w "Primary ATA" same
      line from 8/8 pic2.w "Secondary ATA" same
      "#;
    let diagram = create_diagram(string);
    assert_diagram!(diagram);
  }

  #[test]
  fn ripestat_dependencies_serial() {
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
  fn ripestat_dependencies_parallel() {
    let string = r#"
      path.ui13 up 1in right 3in down 0.5in left 2in down 0.5in left 1in
      "#;
    let diagram = create_diagram(string);
    assert_diagram!(diagram);
  }
}