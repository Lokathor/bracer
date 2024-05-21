use super::*;

pub fn put_fn_in_section_impl(token_stream: TokenStream) -> TokenStream {
  let section_name = one_str_literal_or_panic(token_stream);

  TokenStream::from(TokenTree::Literal(Literal::string(&format!(
    r#".section {section_name},"ax",%progbits"#
  ))))
}
