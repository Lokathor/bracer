use bracer::{
  a32_fake_blx, a32_read_spsr_to, a32_set_cpu_control, a32_write_spsr_from,
  put_fn_in_section, t32_with_a32_scope, when,
};

#[test]
fn test_a32_read_spsr_to() {
  assert_eq!(a32_read_spsr_to!("r0"), "mrs r0, SPSR");
  assert_eq!(a32_read_spsr_to!("R0"), "mrs R0, SPSR");
  assert_eq!(a32_read_spsr_to!("lr"), "mrs lr, SPSR");
  assert_eq!(a32_read_spsr_to!("{temp}"), "mrs {temp}, SPSR");

  unsafe {
    core::arch::asm!(
      // rustfmt stop making this one line
      "/*",
      a32_read_spsr_to!("r0"),
      "*/",
      options(nostack)
    )
  }
}

#[test]
fn test_a32_write_spsr_from() {
  assert_eq!(a32_write_spsr_from!("r0"), "msr r0, SPSR");
  assert_eq!(a32_write_spsr_from!("R0"), "msr R0, SPSR");
  assert_eq!(a32_write_spsr_from!("lr"), "msr lr, SPSR");
  assert_eq!(a32_write_spsr_from!("{temp}"), "msr {temp}, SPSR");
}

#[test]
fn test_a32_fake_blx() {
  let expected = concat!("add lr, pc, #0\n", "bx r12",);
  let actual = a32_fake_blx!("r12");
  assert_eq!(expected, actual);
}

#[test]
fn test_put_fn_in_section() {
  let expected = ".section .text._start,\"ax\",%progbits";
  let actual = put_fn_in_section!(".text._start");
  assert_eq!(expected, actual);
}

#[test]
fn test_a32_set_cpu_control() {
  let expected = "msr CPSR_c, #0b00011111";
  let actual =
    a32_set_cpu_control!(System, irq_masked = false, fiq_masked = false);
  assert_eq!(expected, actual);

  let expected = "msr CPSR_c, #0b10010011";
  let actual =
    a32_set_cpu_control!(Supervisor, irq_masked = true, fiq_masked = false);
  assert_eq!(expected, actual);
}

#[test]
fn test_t32_with_a32_scope() {
  // test that the output works within an `asm!` invocation.
  unsafe {
    core::arch::asm!(
      "/*",
      t32_with_a32_scope!(
        // rustfmt stop making this one line
        "add r0, r0, r0",
        // make sure that we can call other macros within this macro
        a32_read_spsr_to!("r0"),
      ),
      "*/",
      options(nostack)
    )
  }

  // test that 'multi-line' input works (where there's a comma on the end)
  let expected = ".code 32\nmov r0, #0\nadd r0, r0, r0\n.code 16\n";
  let actual = t32_with_a32_scope!(
    // rustfmt stop making this one line
    "mov r0, #0",
    "add r0, r0, r0",
  );
  assert_eq!(expected, actual);

  // test that 'one line' of input works (with no comma on the end)
  let expected = ".code 32\nadd r0, r0, r0\n.code 16\n";
  let actual = t32_with_a32_scope!("add r0, r0, r0");
  assert_eq!(expected, actual);

  // test that the macro works on an empty input sequence.
  let expected = ".code 32\n.code 16\n";
  let actual = t32_with_a32_scope!();
  assert_eq!(expected, actual);
}

#[test]
fn test_when() {
  let expected = concat!(
    "cmp r0, #0\n",
    "beq 1f\n",
    "add r1, r2, r3\n",
    "add r0, r1, r4\n",
    "1:\n"
  );
  let actual = when!(("r0" != "#0")[1]{
    "add r1, r2, r3",
    "add r0, r1, r4",
  });
  assert_eq!(expected, actual);

  // signedness doesn't matter
  let _actual = when!(("r0" == "#0")[1]{
    "add r1, r2, r3",
    "add r0, r1, r4",
  });
  let _actual = when!(("r0" != "#0")[1]{
    "add r1, r2, r3",
    "add r0, r1, r4",
  });

  // unsigned
  let _actual = when!(("r0" >=u "#0")[1]{
    "add r1, r2, r3",
    "add r0, r1, r4",
  });
  let _actual = when!(("r0" <=u "#0")[1]{
    "add r1, r2, r3",
    "add r0, r1, r4",
  });
  let _actual = when!(("r0" <u "#0")[1]{
    "add r1, r2, r3",
    "add r0, r1, r4",
  });
  let _actual = when!(("r0" >u "#0")[1]{
    "add r1, r2, r3",
    "add r0, r1, r4",
  });

  // signed
  let _actual = when!(("r0" >=i "#0")[1]{
    "add r1, r2, r3",
    "add r0, r1, r4",
  });
  let _actual = when!(("r0" <=i "#0")[1]{
    "add r1, r2, r3",
    "add r0, r1, r4",
  });
  let _actual = when!(("r0" <i "#0")[1]{
    "add r1, r2, r3",
    "add r0, r1, r4",
  });
  let _actual = when!(("r0" >i "#0")[1]{
    "add r1, r2, r3",
    "add r0, r1, r4",
  });
}
