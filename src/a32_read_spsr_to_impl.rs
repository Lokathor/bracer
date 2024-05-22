use super::*;

pub fn a32_read_spsr_to_impl(token_stream: TokenStream) -> TokenStream {
  let reg_name = one_str_literal_or_panic(token_stream);

  TokenStream::from(TokenTree::Literal(Literal::string(&format!(
    "mrs {reg_name}, SPSR"
  ))))
}
