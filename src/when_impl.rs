use super::*;

pub fn when_impl(token_stream: TokenStream) -> TokenStream {
  let mut token_iter = token_stream.into_iter();
  let test_group = get_group(token_iter.next().expect("too few tokens"))
    .expect("must have a group for the test");
  let body_group = get_group(token_iter.next().expect("too few tokens"))
    .expect("must have a group for the body");
  assert!(token_iter.next().is_none(), "too many tokens");
  let mut out_buffer: Vec<TokenTree> = Vec::new();
  let local_label = next_local_label();

  let test_trees: Vec<EzTokenTree> =
    test_group.stream().into_iter().map(EzTokenTree::from).collect();
  use EzTokenTree::*;
  use Spacing::*;
  // We're branching when the test *does not* pass, so for example when the
  // users passes in `==` we branch using the inverted case's condition, `ne`
  #[allow(unused_variables)]
  let cond = match test_trees.as_slice() {
    // equality has no signed-ness
    [EzLi(lhs), EzPu('=', Joint), EzPu('=', _), EzLi(op2)] => "ne",
    [EzLi(lhs), EzPu('!', Joint), EzPu('=', _), EzLi(op2)] => "eq",

    // unsigned comparison
    [EzLi(lhs), EzPu('>', Joint), EzPu('=', _), EzId(i, _), EzLi(op2)]
      if i == "u" =>
    {
      "lo"
    }
    [EzLi(lhs), EzPu('<', Joint), EzPu('=', _), EzId(u, _), EzLi(op2)]
      if u == "u" =>
    {
      "hi"
    }
    [EzLi(lhs), EzPu('<', Alone), EzId(u, _), EzLi(op2)] if u == "u" => "hs",
    [EzLi(lhs), EzPu('>', Alone), EzId(u, _), EzLi(op2)] if u == "u" => "ls",

    // signed comparison
    [EzLi(lhs), EzPu('>', Joint), EzPu('=', _), EzId(i, _), EzLi(op2)]
      if i == "i" =>
    {
      "lt"
    }
    [EzLi(lhs), EzPu('<', Joint), EzPu('=', _), EzId(i, _), EzLi(op2)]
      if i == "i" =>
    {
      "gt"
    }
    [EzLi(lhs), EzPu('<', Alone), EzId(i, _), EzLi(op2)] if i == "i" => "ge",
    [EzLi(lhs), EzPu('>', Alone), EzId(i, _), EzLi(op2)] if i == "i" => "le",
    _ => panic!("unknown test expression"),
  };
  let lhs = test_trees.first().unwrap().get_literal().unwrap();
  let op2 = test_trees.last().unwrap().get_literal().unwrap();
  out_buffer.push(TokenTree::Literal(Literal::string(&format!(
    "cmp {lhs}, {op2}
    b{cond} {local_label}\n"
  ))));
  out_buffer.push(TokenTree::Punct(Punct::new(',', Alone)));

  for token_tree in body_group.stream() {
    match token_tree {
      TokenTree::Punct(p) if p == ',' => {
        out_buffer.push(TokenTree::Punct(Punct::new(',', Alone)));
        out_buffer.push(TokenTree::Literal(Literal::character('\n')));
        out_buffer.push(TokenTree::Punct(Punct::new(',', Alone)));
      }
      _ => {
        out_buffer.push(token_tree);
      }
    }
  }

  // check for a trailing comma in our group, if we DO NOT see one then we have
  // to apply a fix before placing the ending label.
  if !matches!(out_buffer.last().unwrap(), TokenTree::Punct(p) if *p == ',') {
    out_buffer.push(TokenTree::Punct(Punct::new(',', Alone)));
    out_buffer.push(TokenTree::Literal(Literal::character('\n')));
    out_buffer.push(TokenTree::Punct(Punct::new(',', Alone)));
  }
  out_buffer
    .push(TokenTree::Literal(Literal::string(&format!("{local_label}:\n"))));

  let concat_expr = vec![
    TokenTree::Ident(Ident::new("concat", Span::call_site())),
    TokenTree::Punct(Punct::new('!', Alone)),
    TokenTree::Group(Group::new(
      Delimiter::Parenthesis,
      TokenStream::from_iter(out_buffer),
    )),
  ];

  TokenStream::from_iter(concat_expr)
}
