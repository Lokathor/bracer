use bracer::{
  a32_fake_blx, a32_within_t32, read_spsr, set_cpu_control, write_spsr,
};

#[test]
fn test_read_spsr() {
  assert_eq!(read_spsr!("r0"), "mrs r0, SPSR");
  assert_eq!(read_spsr!("R0"), "mrs R0, SPSR");
  assert_eq!(read_spsr!("lr"), "mrs lr, SPSR");

  unsafe {
    core::arch::asm!(
      // rustfmt stop making this one line
      "/*",
      read_spsr!("r0"),
      "*/",
      options(nostack)
    )
  }
}

#[test]
fn test_write_spsr() {
  assert_eq!(write_spsr!("r0"), "msr r0, SPSR");
  assert_eq!(write_spsr!("R0"), "msr R0, SPSR");
  assert_eq!(write_spsr!("lr"), "msr lr, SPSR");
}

#[test]
fn test_a32_fake_blx() {
  let asm_lines = a32_fake_blx!("r0");
  let mut lines = asm_lines.lines();
  let mut line = lines.next().unwrap();
  let mut line_iter = line.split(' ');

  assert_eq!(line_iter.next().unwrap(), "adr");
  assert_eq!(line_iter.next().unwrap(), "lr,");
  let local_label = line_iter.next().unwrap();
  assert!(line_iter.next().is_none());

  line = lines.next().unwrap();
  assert_eq!(line.trim(), "bx r0");

  line = lines.next().unwrap();
  assert_eq!(line.trim(), format!("{local_label}:"));

  assert!(lines.next().is_none());
}

#[test]
fn test_a32_within_t32() {
  // test that the output works within an `asm!` invocation.
  unsafe {
    core::arch::asm!(
      "/*",
      a32_within_t32!(
        // rustfmt stop making this one line
        "add r0, r0, r0",
        // make sure that we can call other macros within this macro
        read_spsr!("r0"),
      ),
      "*/",
      options(nostack)
    )
  }

  // test that 'multi-line' input works (where there's a comma on the end)
  let expected = ".code 32\nmov r0, #0\nadd r0, r0, r0\n.code 16\n";
  let actual = a32_within_t32!(
    // rustfmt stop making this one line
    "mov r0, #0",
    "add r0, r0, r0",
  );
  assert_eq!(expected, actual);

  // test that 'one line' of input works (with no comma on the end)
  let expected = ".code 32\nadd r0, r0, r0\n.code 16\n";
  let actual = a32_within_t32!("add r0, r0, r0");
  assert_eq!(expected, actual);

  // test that the macro works on an empty input sequence.
  let expected = ".code 32\n.code 16\n";
  let actual = a32_within_t32!();
  assert_eq!(expected, actual);
}

#[test]
fn test_set_cpu_control() {
  let expected = "msr CPSR_c, #0b00011111";
  let actual = set_cpu_control!(System, irq_masked: false, fiq_masked: false);
  assert_eq!(expected, actual);

  let expected = "msr CPSR_c, #0b10010011";
  let actual =
    set_cpu_control!(Supervisor, irq_masked: true, fiq_masked: false);
  assert_eq!(expected, actual);
}
