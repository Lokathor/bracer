use super::*;

pub fn a32_within_t32_impl(token_stream: TokenStream) -> TokenStream {
  let mut out_buffer: Vec<TokenTree> = Vec::new();
  out_buffer.push(TokenTree::Literal(Literal::string(".code 32\n")));
  out_buffer.push(TokenTree::Punct(Punct::new(',', Spacing::Alone)));

  extend_concat_as_lines(&mut out_buffer, token_stream);
  // the above fn always leaves a trailing comma, no need for a secondary check.
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
