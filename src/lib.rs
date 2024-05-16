extern crate proc_macro;
use core::sync::atomic::{AtomicU64, Ordering};

use proc_macro::{Literal, TokenStream, TokenTree};

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

fn get_str_literal(tree: &TokenTree) -> Option<String> {
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

fn next_u64() -> u64 {
  static NEXT: AtomicU64 = AtomicU64::new(0);
  NEXT.fetch_add(1, Ordering::Relaxed)
}
fn next_local_label() -> String {
  format!(".L_bracer_local_label_{}", next_u64())
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
  let reg_name = get_str_literal(tree).expect("input must be a string literal");
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
  let reg_name = get_str_literal(tree).expect("input must be a string literal");
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
  let reg_name = get_str_literal(tree).expect("input must be a string literal");
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
