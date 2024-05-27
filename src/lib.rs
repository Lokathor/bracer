#![warn(missing_docs)]
#![allow(unused_imports)]
#![allow(unused_mut)]

//! Various macros to help write inline assembly for ARM targets.
//!
//! These macros help you get your assembly written, but they have nearly no
//! ability to help ensure that your assembly is correct. In rare cases where
//! something can be statically known to be "obviously" wrong (eg: an invalid
//! register name is picked for a specific instruction) the macro will panic.

extern crate proc_macro;
use core::{
  fmt::Write,
  str::FromStr,
  sync::atomic::{AtomicU64, Ordering},
};
use proc_macro::{
  Delimiter, Group, Ident, Literal, Punct, Spacing, Span, TokenStream,
  TokenTree,
};
use util::*;

mod a32_fake_blx_impl;
mod a32_read_spsr_to_impl;
mod a32_set_cpu_control_impl;
mod a32_write_spsr_from_impl;
mod put_fn_in_section_impl;
mod t32_with_a32_scope_impl;
mod util;
mod when_impl;

/// Reads SPSR to the register given.
///
/// ## Input
/// A single string literal that's an actual register name (eg: `"r0"`), or an
/// assembly register substitution name (eg: `"{temp}"`).
///
/// ## Output
/// This expands to one line of assembly using the [`mrs`][mrs_docs] instruction
/// to read SPSR to the named register.
///
/// ## Assembly Safety
/// * From the `mrs` Docs: *You must not attempt to access the SPSR when the
///   processor is in User or System mode. This is your responsibility. The
///   assembler cannot warn you about this, because it has no information about
///   the processor mode at execution time.*
///
/// [mrs_docs]: https://developer.arm.com/documentation/dui0473/m/arm-and-thumb-instructions/mrs--system-coprocessor-register-to-arm-register-
#[proc_macro]
pub fn a32_read_spsr_to(token_stream: TokenStream) -> TokenStream {
  a32_read_spsr_to_impl::a32_read_spsr_to_impl(token_stream)
}

/// Writes SPSR from the register given.
///
/// ## Input
/// A single string literal that's an actual register name (eg: `"r0"`), or an
/// assembly register substitution name (eg: `"{temp}"`).
///
/// ## Output
/// This expands to one line of assembly using the [`msr`][msr_docs] instruction
/// to write SPSR from the named register.
///
/// ## Assembly Safety
/// * The `mrs` docs warn you not to use `mrs` to access SPSR when in User or
///   System mode. The related `msr` instruction generated by this macro should
///   likely *also* not be used when in User or System mode. See
///   [`a32_read_spsr_to!`].
///
/// [msr_docs]: https://developer.arm.com/documentation/dui0489/i/arm-and-thumb-instructions/msr--arm-register-to-system-coprocessor-register-
#[proc_macro]
pub fn a32_write_spsr_from(token_stream: TokenStream) -> TokenStream {
  a32_write_spsr_from_impl::a32_write_spsr_from_impl(token_stream)
}

/// ARMv4T lacks the actual `blx` instruction, so this performs a "fake"
/// `blx`-styled operation.
///
/// ## Input
/// A single string literal that's an actual register name (eg: `"r0"`), or an
/// assembly register substitution name (eg: `"{temp}"`).
///
/// ## Output
/// Emits a string literal of `a32` code like the following:
/// ```arm
/// add lr, pc, #0 // set `lr` to just after the `bx`
/// bx <reg>
/// ```
///
/// This assembly is only correct in `a32` state.
#[proc_macro]
pub fn a32_fake_blx(token_stream: TokenStream) -> TokenStream {
  a32_fake_blx_impl::a32_fake_blx_impl(token_stream)
}

/// Emits a `.section` directive to place the code in a section name you pick.
///
/// Use this *before* the label for the function you're writing.
///
/// ## Input
/// A string literal that's a valid section name. It should be alphanumeric,
/// also `.` is allowed.
///
/// ## Output
/// Emits a `.section` directive with the section name you specify and also
/// properly marks the section as `allocated` and `executable`.
#[proc_macro]
pub fn put_fn_in_section(token_stream: TokenStream) -> TokenStream {
  put_fn_in_section_impl::put_fn_in_section_impl(token_stream)
}

/// Places `.code 32` at the start and `.code 16` at the end of the input
/// sequence.
///
/// ## Input
/// The input sequence should be zero or more expressions (comma separated) that
/// could each normally be used within an `asm!` block.
///
/// ## Output
/// A single `concat!` expression, with newlines inserted for each input
/// expression, and with the `.code` directives at the start and end.
///
/// ## Safety
/// You **must* use this within `t32` code only. You **must not** use this
/// within an `a32` encoded assembly block. It will leave the assembler in a bad
/// state after the assembly string, which is UB.
#[proc_macro]
pub fn t32_with_a32_scope(token_stream: TokenStream) -> TokenStream {
  t32_with_a32_scope_impl::t32_with_a32_scope_impl(token_stream)
}

/// Generates the asm string to set the CPU control bits.
///
/// Input must be of the form:
/// ```text
/// {mode_name}, irq_masked = {bool}, fiq_masked = {bool}
/// ```
///
/// Valid mode names are the long name or short name of a CPU mode:
/// * User / usr
/// * FIQ / fiq
/// * IRQ / irq
/// * Supervisor / svc
/// * System / sys
///
/// ## Assembly Safety
/// This instruction can only be used in `a32` code.
#[proc_macro]
pub fn a32_set_cpu_control(token_stream: TokenStream) -> TokenStream {
  a32_set_cpu_control_impl::a32_set_cpu_control_impl(token_stream)
}

/// Emits code that will perform the test and skip past some lines if the test
/// does not pass.
///
/// **Usage Example:**
/// ```rust
/// # use bracer::*;
/// # let s =
/// when!(("r0" != "#0")[1]{
///   "add r1, r2, r3",
///   "add r0, r1, r4",
/// })
/// # ;
/// ```
///
/// * The test to perform must be in one grouping.
/// * The number literal for the numeric label placed at the end of the block
///   must be another grouping.
/// * The lines to execute when the test passes must be in a separate grouping.
/// * The macro *does not* care what grouping markers you use, `()`, `[]`, and
///   `{}` are all fine.
#[proc_macro]
pub fn when(token_stream: TokenStream) -> TokenStream {
  when_impl::when_impl(token_stream)
}
