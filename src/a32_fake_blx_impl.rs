use super::*;

pub fn a32_fake_blx_impl(token_stream: TokenStream) -> TokenStream {
  let reg_name = one_str_literal_or_panic(token_stream);

  TokenStream::from(TokenTree::Literal(Literal::string(&format!(
    "add lr, pc, #0\nbx {reg_name}"
  ))))
}
