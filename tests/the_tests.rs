use bracer::{
  a32_fake_blx, a32_within_t32, put_fn_in_section, read_spsr_to,
  set_cpu_control, when, write_spsr_from,
};

#[test]
fn test_read_spsr_to() {
  assert_eq!(read_spsr_to!("r0"), "mrs r0, SPSR");
  assert_eq!(read_spsr_to!("R0"), "mrs R0, SPSR");
  assert_eq!(read_spsr_to!("lr"), "mrs lr, SPSR");
  assert_eq!(read_spsr_to!("{temp}"), "mrs {temp}, SPSR");

  unsafe {
    core::arch::asm!(
      // rustfmt stop making this one line
      "/*",
      read_spsr_to!("r0"),
      "*/",
      options(nostack)
    )
  }
}

#[test]
fn test_write_spsr_from() {
  assert_eq!(write_spsr_from!("r0"), "msr r0, SPSR");
  assert_eq!(write_spsr_from!("R0"), "msr R0, SPSR");
  assert_eq!(write_spsr_from!("lr"), "msr lr, SPSR");
  assert_eq!(write_spsr_from!("{temp}"), "msr {temp}, SPSR");
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
        read_spsr_to!("r0"),
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

#[test]
fn test_put_fn_in_section() {
  let expected = ".section .iwram._start,\"ax\",%progbits";
  let actual = put_fn_in_section!(".iwram._start");
  assert_eq!(expected, actual);
}

#[test]
fn test_when() {
  let expected = "cmp reg, op2
    b{cond} to label
    add r1, r2, r3
    add r0, r1, r4
    .L_local_label_here:";
  let actual = when!(("r0" != "#0"){
    "add r1, r2, r3",
    "add r0, r1, r4",
  });
  assert_eq!(expected.lines().count(), actual.lines().count());
}
