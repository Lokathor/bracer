extern crate proc_macro;
use proc_macro::{Literal, TokenStream, TokenTree};

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
  const ALLOWED_TARGET_REG_LIST: &[&str] = &[
    "r0", "R0", "r1", "R1", "r2", "R2", "r3", "R3", "r4", "R4", "r5", "R5",
    "r6", "R6", "r7", "R7", "r8", "R8", "r9", "R9", "r10", "R10", "r11", "R11",
    "r12", "R12", "r14", "R14", "lr", "LR",
  ];
  match tree {
    TokenTree::Group(_) => panic!("expected str literal, found group"),
    TokenTree::Punct(_) => {
      panic!("expected ident or str literal, found punctuation")
    }
    TokenTree::Ident(_) => {
      panic!("expected ident or str literal, found identifier")
    }
    TokenTree::Literal(l) => {
      let string = format!("{l}");
      let without_quotes = &string[..string.len() - 1][1..];
      if !ALLOWED_TARGET_REG_LIST.contains(&without_quotes) {
        panic!("register name `{without_quotes}` is not on the permitted list")
      }
      let output = format!("mrs {without_quotes}, SPSR");

      TokenStream::from(TokenTree::Literal(Literal::string(&output)))
    }
  }
}
