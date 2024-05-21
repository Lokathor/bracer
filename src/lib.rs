#![warn(missing_docs)]
#![allow(unused_imports)]
#![allow(unused_mut)]

//! Various macros to help write inline assembly for ARM targets.
//!
//! These macros help you get your assembly written, but they have nearly no
//! ability to help ensure that your assembly is correct. In rare cases where
//! something can be statically known to be "obviously" wrong (eg: an invalid
//! register name is picked for a specific instruction) the macro will panic.

/* TODO: with_pushed_registers */

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

mod a32_fake_blx_impl;
mod put_fn_in_section_impl;
mod read_spsr_to_impl;
mod util;
mod write_spsr_from_impl;
use util::*;

/// Generates a unique "local" label string.
fn next_local_label() -> String {
  static NEXT: AtomicU64 = AtomicU64::new(0);
  format!(".L_bracer_local_label_{}", NEXT.fetch_add(1, Ordering::Relaxed))
}

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
pub fn read_spsr_to(token_stream: TokenStream) -> TokenStream {
  read_spsr_to_impl::read_spsr_to_impl(token_stream)
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
///   [`read_spsr_to`].
///
/// [msr_docs]: https://developer.arm.com/documentation/dui0489/i/arm-and-thumb-instructions/msr--arm-register-to-system-coprocessor-register-
#[proc_macro]
pub fn write_spsr_from(token_stream: TokenStream) -> TokenStream {
  write_spsr_from_impl::write_spsr_from_impl(token_stream)
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
/// adr lr .L_bracer_local_label_<id>
/// bx <reg>
/// .L_bracer_local_label_<id>:
/// ```
///
/// Every expansion uses a uniquely generated local label id, so this shouldn't
/// ever clash with any other code.
///
/// This can only be used in `a32` state, because you can't move values directly
/// to `lr` in `t32` state.
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
/// **Usage Example:**
/// ```
/// # use bracer::*;
/// # let s =
/// a32_within_t32! {
///   "mov r0, #0",
///   "str r1, [r0]",
/// }
/// # ;
/// ```
///
/// The input sequence should be zero or more expressions (comma separated) that
/// could each normally be used within an `asm!` block. The output is a single
/// `concat!` expression, with newlines inserted for each input expression, and
/// with the `.code` directives at the start and end.
///
/// ## Safety
/// You **must not** use this within an `a32` encoded assembly block. It will
/// leave the assembler in a bad state after the assembly string, which is UB.
#[proc_macro]
pub fn a32_within_t32(token_stream: TokenStream) -> TokenStream {
  // Note(Lokathor): The output of this macro has to be "one expression", so we
  // look through the comma separated list of input expressions and then re-emit
  // everything as a single `concat!` expresion. For each input expression we
  // get, we want to put that expression followed by a newline into the output
  // `concat!`. The trick is that we need to be careful about where all the
  // commas are, because having two commas in a row within the `concat!`
  // argument list will cause an error.
  let mut out_buffer: Vec<TokenTree> = Vec::new();
  out_buffer.push(TokenTree::Literal(Literal::string(".code 32\n")));
  out_buffer.push(TokenTree::Punct(Punct::new(',', Spacing::Alone)));
  for token_tree in token_stream {
    // We want the input of this proc-macro to "act like" you were giving input
    // directly to the `asm!` macro. This means that each comma between the
    // input expressions has to act like a newline within the fully concatinated
    // output. Whenever we see a comma in the input, we insert that *and also*
    // we insert a newline character (followed by a comma for the newline
    // character's expression).
    match token_tree {
      TokenTree::Punct(p) if p == ',' => {
        out_buffer.push(TokenTree::Punct(Punct::new(',', Spacing::Alone)));
        out_buffer.push(TokenTree::Literal(Literal::character('\n')));
        out_buffer.push(TokenTree::Punct(Punct::new(',', Spacing::Alone)));
      }
      _ => {
        out_buffer.push(token_tree);
      }
    }
  }
  // check for a trailing comma in our group, if we DO NOT see one then we have
  // to apply a fix before placing the literal for the final `.code 16` line.
  if !matches!(out_buffer.last().unwrap(), TokenTree::Punct(p) if *p == ',') {
    out_buffer.push(TokenTree::Punct(Punct::new(',', Spacing::Alone)));
    out_buffer.push(TokenTree::Literal(Literal::character('\n')));
    out_buffer.push(TokenTree::Punct(Punct::new(',', Spacing::Alone)));
  }
  out_buffer.push(TokenTree::Literal(Literal::string(".code 16\n")));

  let concat_expr = vec![
    TokenTree::Ident(Ident::new("concat", Span::call_site())),
    TokenTree::Punct(Punct::new('!', Spacing::Alone)),
    TokenTree::Group(Group::new(
      Delimiter::Parenthesis,
      TokenStream::from_iter(out_buffer),
    )),
  ];

  TokenStream::from_iter(concat_expr)
}

/// Generates the asm string to set the CPU control bits.
///
/// Input must be of the form: `{mode_name}, irq_masked: {bool}, fiq_masked:
/// {bool}`
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
pub fn set_cpu_control(token_stream: TokenStream) -> TokenStream {
  let mut stream_iter = token_stream.into_iter();
  // CPSR low bits are: `I F T MMMMM`, and T must always be left as 0.
  let mode =
    match stream_iter.next().expect("too few tokens").to_string().as_str() {
      "User" | "usr" => "10000",
      "FIQ" | "fiq" => "10001",
      "IRQ" | "irq" => "10010",
      "Supervisor" | "svc" => "10011",
      "System" | "sys" => "11111",
      other => {
        panic!("First argument must be a valid cpu mode name, got `{other}`")
      }
    };
  assert_eq!(
    stream_iter.next().expect("too few tokens").to_string(),
    ",",
    "must have comma after the first arg"
  );

  assert_eq!(
    stream_iter.next().expect("too few tokens").to_string(),
    "irq_masked",
    "second setting must be `irq_masked`"
  );
  assert_eq!(
    stream_iter.next().expect("too few tokens").to_string(),
    ":",
    "after `irq_masked` must be a `:`"
  );
  let i = get_bool(&stream_iter.next().expect("too few tokens"))
    .expect("`irq_masked` must be set as `true` or `false`") as u8;
  assert_eq!(
    stream_iter.next().expect("too few tokens").to_string(),
    ",",
    "must have comma after the second arg"
  );

  assert_eq!(
    stream_iter.next().expect("too few tokens").to_string(),
    "fiq_masked",
    "third setting must be `fiq_masked`"
  );
  assert_eq!(
    stream_iter.next().expect("too few tokens").to_string(),
    ":",
    "after `fiq_masked` must be a `:`"
  );
  let f = get_bool(&stream_iter.next().expect("too few tokens"))
    .expect("`fiq_masked` must be set as `true` or `false`") as u8;
  assert!(stream_iter.next().is_none(), "too many tokens");
  TokenStream::from_iter(Some(TokenTree::Literal(Literal::string(&format!(
    "msr CPSR_c, #0b{i}{f}0{mode}"
  )))))
}

/// Emits code that will perform the test and skip past some lines if the test
/// does not pass.
///
/// **Usage Example:**
/// ```rust
/// # use bracer::*;
/// # let s =
/// when!(("r0" != "#0"){
///   "add r1, r2, r3",
///   "add r0, r1, r4",
/// })
/// # ;
/// ```
///
/// * The test to perform must be in one grouping.
/// * The lines to execute when the test passes must be in a separate grouping.
/// * The macro *does not* care what grouping markers you use, `()`, `[]`, and
///   `{}` are all fine.
#[proc_macro]
pub fn when(token_stream: TokenStream) -> TokenStream {
  let mut token_iter = token_stream.into_iter();
  let test_group = get_group(token_iter.next().expect("too few tokens"))
    .expect("must have a group for the test");
  let body_group = get_group(token_iter.next().expect("too few tokens"))
    .expect("must have a group for the body");
  assert!(token_iter.next().is_none(), "too many tokens");
  let mut out_buffer: Vec<TokenTree> = Vec::new();
  let local_label = next_local_label();

  let test_trees: Vec<TokenTree> = test_group.stream().into_iter().collect();
  match test_trees.len() {
    4 => {
      let test0 = get_str_literal_content(&test_trees[0])
        .expect("test0 must be a str literal");
      let test1 =
        get_punct(test_trees[1].clone()).expect("test1 must be a punctuation");
      let test2 =
        get_punct(test_trees[2].clone()).expect("test2 must be a punctuation");
      let test3 = get_str_literal_content(&test_trees[3])
        .expect("test3 must be a str literal");
      if test1 == '!' && test2 == '=' {
        out_buffer.push(TokenTree::Literal(Literal::string(&format!(
          "cmp {test0}, {test3}\n"
        ))));
        out_buffer.push(TokenTree::Punct(Punct::new(',', Spacing::Alone)));
        out_buffer.push(TokenTree::Literal(Literal::string(&format!(
          "bne {local_label}\n"
        ))));
        out_buffer.push(TokenTree::Punct(Punct::new(',', Spacing::Alone)));
      } else {
        panic!("unknown test expression")
      }
    }
    other => panic!("bad number of test tokens: `{other}`"),
  }

  for token_tree in body_group.stream() {
    match token_tree {
      TokenTree::Punct(p) if p == ',' => {
        out_buffer.push(TokenTree::Punct(Punct::new(',', Spacing::Alone)));
        out_buffer.push(TokenTree::Literal(Literal::character('\n')));
        out_buffer.push(TokenTree::Punct(Punct::new(',', Spacing::Alone)));
      }
      _ => {
        out_buffer.push(token_tree);
      }
    }
  }

  // check for a trailing comma in our group, if we DO NOT see one then we have
  // to apply a fix before placing the ending label.
  if !matches!(out_buffer.last().unwrap(), TokenTree::Punct(p) if *p == ',') {
    out_buffer.push(TokenTree::Punct(Punct::new(',', Spacing::Alone)));
    out_buffer.push(TokenTree::Literal(Literal::character('\n')));
    out_buffer.push(TokenTree::Punct(Punct::new(',', Spacing::Alone)));
  }
  out_buffer
    .push(TokenTree::Literal(Literal::string(&format!("{local_label}\n"))));

  let concat_expr = vec![
    TokenTree::Ident(Ident::new("concat", Span::call_site())),
    TokenTree::Punct(Punct::new('!', Spacing::Alone)),
    TokenTree::Group(Group::new(
      Delimiter::Parenthesis,
      TokenStream::from_iter(out_buffer),
    )),
  ];

  TokenStream::from_iter(concat_expr)
}
