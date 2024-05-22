use super::*;

pub fn a32_write_spsr_from_impl(token_stream: TokenStream) -> TokenStream {
  let reg_name = one_str_literal_or_panic(token_stream);

  TokenStream::from(TokenTree::Literal(Literal::string(&format!(
    "msr {reg_name}, SPSR"
  ))))
}
