use super::*;

pub fn a32_fake_blx_impl(token_stream: TokenStream) -> TokenStream {
  let reg_name = one_str_literal_or_panic(token_stream);

  let label = next_local_label();
  TokenStream::from(TokenTree::Literal(Literal::string(&format!(
    "adr lr, {label}
    bx {reg_name}
    {label}:"
  ))))
}
