#![allow(unused_imports)]
#![allow(unused_mut)]

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

const SPSR_ALLOWED_TARGET_REG_LIST: &[&str] = &[
  "r0", "R0", "r1", "R1", "r2", "R2", "r3", "R3", "r4", "R4", "r5", "R5", "r6",
  "R6", "r7", "R7", "r8", "R8", "r9", "R9", "r10", "R10", "r11", "R11", "r12",
  "R12", "r14", "R14", "lr", "LR",
];

const ANY_REG_NAME: &[&str] = &[
  "r0", "R0", "r1", "R1", "r2", "R2", "r3", "R3", "r4", "R4", "r5", "R5", "r6",
  "R6", "r7", "R7", "r8", "R8", "r9", "R9", "r10", "R10", "r11", "R11", "r12",
  "R12", "r13", "R13", "r14", "R14", "r15", "R15", "lr", "LR", "pc", "PC",
];

/// Strips the double quotes from a string literal.
///
/// ## Failure
/// * If the input isn't a `Literal`, or it is a `Literal` but doesn't start and
///   end with `"` when formatted, then you get `None`.
fn get_str_literal_content(tree: &TokenTree) -> Option<String> {
  match tree {
    TokenTree::Group(_) | TokenTree::Ident(_) | TokenTree::Punct(_) => None,
    TokenTree::Literal(l) => {
      let string = format!("{l}");
      if string.starts_with('"') && string.ends_with('"') {
        Some(string[..string.len() - 1][1..].to_string())
      } else {
        None
      }
    }
  }
}

/// Generates a unique "local" label string.
fn next_local_label() -> String {
  static NEXT: AtomicU64 = AtomicU64::new(0);
  format!(".L_bracer_local_label_{}", NEXT.fetch_add(1, Ordering::Relaxed))
}

/// Reads SPSR to a named register.
///
/// This evaluates to a string literal for the following assembly:
///
/// ```arm
/// mrs <reg>, SPSR   // 'move register <-- special'
/// ```
///
/// Input should be a string literal naming the register you want to store the
/// SPSR value in.
///
/// ## Panics
/// * If the target register isn't allowed to be used with the `mrs`
///   instruction, this will panic. You can use `r0` through `r12`, and `lr`
///   (aka `r14`). You can also use any of those registers written with
///   uppercase.
///
/// ## Assembly Safety
/// * From the [ARM Docs]: *You must not attempt to access the SPSR when the
///   processor is in User or System mode. This is your responsibility. The
///   assembler cannot warn you about this, because it has no information about
///   the processor mode at execution time.*
///
/// [ARM Docs]:
///     https://developer.arm.com/documentation/dui0489/i/arm-and-thumb-instructions/mrs--psr-to-general-purpose-register-
#[proc_macro]
pub fn read_spsr(token_stream: TokenStream) -> TokenStream {
  let trees: Vec<TokenTree> = token_stream.into_iter().collect();
  let tree = match &trees[..] {
    [tree] => tree,
    _ => panic!("please provide only a single ident or string literal"),
  };
  let reg_name =
    get_str_literal_content(tree).expect("input must be a string literal");
  if !SPSR_ALLOWED_TARGET_REG_LIST.contains(&reg_name.as_str()) {
    panic!("register name `{reg_name}` is not on the permitted list")
  }
  let output = format!("mrs {reg_name}, SPSR");

  TokenStream::from(TokenTree::Literal(Literal::string(&output)))
}

/// Writes SPSR from a named register.
///
/// This evaluates to a string literal for the following assembly:
///
/// ```arm
/// msr <reg>, SPSR   // 'move special <-- register'
/// ```
///
/// Input should be a string literal naming the register that holds the value
/// you want to write to SPSR.
///
/// ## Panics
/// * If the target register isn't allowed to be used with the `msr`
///   instruction, this will panic. You can use `r0` through `r12`, and `lr`
///   (aka `r14`). You can also use any of those registers written with
///   uppercase.
///
/// ## Assembly Safety
/// * From the [ARM Docs]: *You must not attempt to access the SPSR when the
///   processor is in User or System mode. This is your responsibility. The
///   assembler cannot warn you about this, because it has no information about
///   the processor mode at execution time.*
///
/// [ARM Docs]:
///     https://developer.arm.com/documentation/dui0489/i/arm-and-thumb-instructions/mrs--psr-to-general-purpose-register-
#[proc_macro]
pub fn write_spsr(token_stream: TokenStream) -> TokenStream {
  let trees: Vec<TokenTree> = token_stream.into_iter().collect();
  let tree = match &trees[..] {
    [tree] => tree,
    _ => panic!("please provide only a single ident or string literal"),
  };
  let reg_name =
    get_str_literal_content(tree).expect("input must be a string literal");
  if !SPSR_ALLOWED_TARGET_REG_LIST.contains(&reg_name.as_str()) {
    panic!("register name `{reg_name}` is not on the permitted list")
  }
  let output = format!("msr {reg_name}, SPSR");

  TokenStream::from(TokenTree::Literal(Literal::string(&output)))
}

/// Fakes a `blx` type of operation.
///
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
  let trees: Vec<TokenTree> = token_stream.into_iter().collect();
  let tree = match &trees[..] {
    [tree] => tree,
    _ => panic!("please provide only a single ident or string literal"),
  };
  let reg_name =
    get_str_literal_content(tree).expect("input must be a string literal");
  if !ANY_REG_NAME.contains(&reg_name.as_str()) {
    panic!("register name `{reg_name}` is not on the permitted list")
  }
  let label = next_local_label();
  let output = format!(
    "adr lr, {label}
    bx {reg_name}
    {label}:"
  );

  TokenStream::from(TokenTree::Literal(Literal::string(&output)))
}

/// Places `.code 32` at the start and `.code 16` at the end of the input
/// sequence.
///
/// The input sequence should be zero or more expressions (comma separated) that
/// could each normally be used within an `asm!` block.
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
  let mut group_buffer: Vec<TokenTree> = Vec::new();
  group_buffer.push(TokenTree::Literal(Literal::string(".code 32\n")));
  group_buffer.push(TokenTree::Punct(Punct::new(',', Spacing::Alone)));
  for token_tree in token_stream {
    // We want the input of this proc-macro to "act like" you were giving input
    // directly to the `asm!` macro. This means that each comma between the
    // input expressions has to act like a newline within the fully concatinated
    // output. Whenever we see a comma in the input, we insert that *and also*
    // we insert a newline character (followed by a comma for the newline
    // character's expression).
    match token_tree {
      TokenTree::Punct(p) if p == ',' => {
        group_buffer.push(TokenTree::Punct(Punct::new(',', Spacing::Alone)));
        group_buffer.push(TokenTree::Literal(Literal::character('\n')));
        group_buffer.push(TokenTree::Punct(Punct::new(',', Spacing::Alone)));
      }
      _ => {
        group_buffer.push(token_tree);
      }
    }
  }
  // check for a trailing comma in our group, if we DO NOT see one then we have
  // to apply a fix before placing the literal for the final `.code 16` line.
  if !matches!(group_buffer.last().unwrap(), TokenTree::Punct(p) if *p == ',') {
    group_buffer.push(TokenTree::Punct(Punct::new(',', Spacing::Alone)));
    group_buffer.push(TokenTree::Literal(Literal::character('\n')));
    group_buffer.push(TokenTree::Punct(Punct::new(',', Spacing::Alone)));
  }
  group_buffer.push(TokenTree::Literal(Literal::string(".code 16\n")));

  let concat_expr = vec![
    TokenTree::Ident(Ident::new("concat", Span::call_site())),
    TokenTree::Punct(Punct::new('!', Spacing::Alone)),
    TokenTree::Group(Group::new(
      Delimiter::Parenthesis,
      TokenStream::from_iter(group_buffer),
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
  let i = match stream_iter.next().expect("too few tokens").to_string().as_str()
  {
    "true" => "1",
    "false" => "0",
    _ => panic!("`irq_masked` must be set as `true` or `false`"),
  };
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
  let f = match stream_iter.next().expect("too few tokens").to_string().as_str()
  {
    "true" => "1",
    "false" => "0",
    _ => panic!("`fiq_masked` must be set as `true` or `false`"),
  };
  assert!(stream_iter.next().is_none(), "too many tokens");
  TokenStream::from_iter(Some(TokenTree::Literal(Literal::string(&format!(
    "msr CPSR_c, #0b{i}{f}0{mode}"
  )))))
}
