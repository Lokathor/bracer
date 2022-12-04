#![no_std]

/// Builds an assembly string wrapped in `.code 32` and `.code 16` as necessary
///
/// ```txt
/// emit_a32_code!{
///   "lines"
///   "go"
///   "here"
/// };
/// ```
#[macro_export]
#[cfg(target_feature = "thumb-mode")]
macro_rules! emit_a32_code {
  ($($asm_line:expr),+ $(,)?) => {
    concat!(
      concat!(".code 32\n"),

      $( concat!($asm_line, "\n") ),+ ,

      concat!(".code 16\n"),
    )
  }
}

/// Builds an assembly string wrapped in `.code 32` and `.code 16` as necessary
///
/// ```txt
/// emit_a32_code!{
///   "lines"
///   "go"
///   "here"
/// };
/// ```
#[macro_export]
#[cfg(not(target_feature = "thumb-mode"))]
macro_rules! emit_a32_code {
  ($($asm_line:expr),+ $(,)?) => {
    concat!(
      $( concat!($asm_line, "\n") ),+ ,
    )
  }
}

#[test]
fn test_emit_a32_code() {
  #[cfg(target_feature = "thumb-mode")]
  let expected = ".code 32\nmov r0, #0\n.code 16\n";

  #[cfg(not(target_feature = "thumb-mode"))]
  let expected = "mov r0, #0\n";

  let actual = emit_a32_code!("mov r0, #0");
  assert_eq!(expected, actual);
}

/// Builds an assembly string that puts the contained code in the section
/// specified.
///
/// ```txt
/// put_code_in_section!( ".example.section", {
///   "lines"
///   "go"
///   "here"
/// });
/// ```
#[macro_export]
macro_rules! put_code_in_section {
  ($section_name:expr, {
    $($asm_line:expr),+ $(,)?
  }) => {
    concat!(
      concat!(".section ", $section_name, "\n"),
      $( concat!($asm_line, "\n") ),+ ,
      concat!(".previous\n"),
    )
  }
}

#[test]
fn test_put_code_in_section() {
  let expected = ".section .iwram\nmov r0, #0\n.previous\n";

  let actual = put_code_in_section!(".iwram", { "mov r0, #0" });

  assert_eq!(expected, actual);
}

/// Builds an assembly string that pushes some regs, does the body, then pops
/// the regs.
///
/// The `reglist` expression should include the appropriate level of braces for
/// the enclosing assembly block (two for normal asm, or one for raw asm).
///
/// ```txt
/// with_pushed_registers!( "reglist", {
///   "lines"
///   "go"
///   "here"
/// });
/// ```
#[macro_export]
macro_rules! with_pushed_registers {
  ($reglist:expr, {
    $($asm_line:expr),* $(,)?
  }) => {
    concat!(
      concat!("push ", $reglist, "\n"),
      $( concat!($asm_line, "\n") ),* ,
      concat!("pop ", $reglist, "\n"),
    )
  }
}

#[test]
fn test_with_pushed_registers() {
  let expected = "push {{r0, r1}}\nmov r0, #0\npop {{r0, r1}}\n";

  let actual = with_pushed_registers!("{{r0, r1}}", { "mov r0, #0" });

  assert_eq!(expected, actual);
}

/// Reads SPSR into the register named, does the block, and writes the same
/// register back to SPSR.
#[macro_export]
macro_rules! with_spsr_held_in {
  ($reg:literal, {
    $($asm_line:expr),* $(,)?
  }) => {
    concat!(
      concat!("mrs ", $reg, ", SPSR\n"),
      $( concat!($asm_line, "\n") ),* ,
      concat!("msr SPSR, ", $reg, "\n"),
    )
  }
}

#[test]
fn test_with_spsr_held_in() {
  let expected = "mrs r6, SPSR\nmov r0, #0\nmsr SPSR, r6\n";

  let actual = with_spsr_held_in!("r6", { "mov r0, #0" });

  assert_eq!(expected, actual);
}

/// Sets `lr` to just after the `bx`, then uses `bx` with the given register.
///
/// This generates a label, so pick a `label_id` that won't interfere with any
/// nearby code.
#[macro_export]
macro_rules! adr_lr_then_bx_to {
  (reg=$reg_name:expr, label_id=$label:expr) => {
    concat!(
      concat!("adr lr, ", $label, "f\n"),
      concat!("bx ", $reg_name, "\n"),
      concat!($label, ":\n"),
    )
  };
}

/// Expands to the asm line to set the control bits of CPSR.
///
/// * Can only be used in `a32`
/// * Only sets the control bits, all other bits (eg: flag bits) are unchanged.
///
/// Currently, not all possible patterns are covered by this macro, just the
/// patterns needed by this runtime when it was written. In general, any of the
/// five CPU modes can be combined with irq and fiq masking each being either
/// off or on. If a desired combination is missing just add it.
#[macro_export]
macro_rules! set_cpu_control {
  // CPSR low bits are: `I F T MMMMM`, and T must always be left as 0.
  // * 0b10011: Supervisor (SVC)
  // * 0b11111: System (SYS)
  (System, irq_masked: false, fiq_masked: false) => {
    "msr CPSR_c, #0b00011111\n"
  };
  (Supervisor, irq_masked: true, fiq_masked: false) => {
    "msr CPSR_c, #0b10010010\n"
  };
}

/// Performs the appropriate test, then either runs the block or jumps past it,
/// depending on the test result.
///
/// Currently supports:
/// * `$reg == $op2`
/// * `$reg != $op2`
/// * `$reg >=u $op2`
/// * `$reg <=u $op2`
#[macro_export]
macro_rules! when {
  ($reg:literal == $op2:literal [label_id=$label:literal] {
    $($asm_line:expr),* $(,)?
  }) => {
    concat!(
      concat!("cmp ", $reg, ", ", $op2, "\n"),
      concat!("bne ", $label, "f\n"),
      $( concat!($asm_line, "\n") ),* ,
      concat!($label, ":\n"),
    )
  };
  ($reg:literal != $op2:literal [label_id=$label:literal] {
    $($asm_line:expr),* $(,)?
  }) => {
    concat!(
      concat!("cmp ", $reg, ", ", $op2, "\n"),
      concat!("beq ", $label, "f\n"),
      $( concat!($asm_line, "\n") ),* ,
      concat!($label, ":\n"),
    )
  };
  ($reg:literal >=u $op2:literal [label_id=$label:literal] {
    $($asm_line:expr),* $(,)?
  }) => {
    concat!(
      concat!("cmp ", $reg, ", ", $op2, "\n"),
      concat!("bcc ", $label, "f\n"), // cc: Unsigned LT
      $( concat!($asm_line, "\n") ),* ,
      concat!($label, ":\n"),
    )
  };
  ($reg:literal <=u $op2:literal [label_id=$label:literal] {
    $($asm_line:expr),* $(,)?
  }) => {
    concat!(
      concat!("cmp ", $reg, ", ", $op2, "\n"),
      concat!("bhi ", $label, "f\n"), // hi: Unsigned GT
      $( concat!($asm_line, "\n") ),* ,
      concat!($label, ":\n"),
    )
  };
}

#[test]
fn test_when() {
  let expected = "cmp r6, #32\nbcc 2f\nmov r0, #0\n2:\n";

  let actual = when!("r6" >=u "#32" [label_id=2] { "mov r0, #0" });

  assert_eq!(expected, actual);
}
