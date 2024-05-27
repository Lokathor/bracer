use super::*;

pub fn when_impl(token_stream: TokenStream) -> TokenStream {
  use EzTokenTree::*;
  use Spacing::*;

  let mut token_iter = token_stream.into_iter();
  let test_group = get_group(token_iter.next().expect("too few tokens"))
    .expect("must have a group for the test");
  let label_group = get_group(token_iter.next().expect("too few tokens"))
    .expect("must have a group for the label");
  let body_group = get_group(token_iter.next().expect("too few tokens"))
    .expect("must have a group for the body");
  assert!(token_iter.next().is_none(), "too many tokens");

  let mut out_buffer: Vec<TokenTree> = Vec::new();

  let label_trees: Vec<EzTokenTree> =
    label_group.stream().into_iter().map(EzTokenTree::from).collect();
  let local_label: u32 = match label_trees.as_slice() {
    [EzLi(l)] => {
      let f = l.to_string();
      f.parse::<u32>().expect("literal must be a valid u32")
    }
    _ => {
      panic!("please provide only 1 literal for the label")
    }
  };

  let test_trees: Vec<EzTokenTree> =
    test_group.stream().into_iter().map(EzTokenTree::from).collect();
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
  let lhs = test_trees
    .first()
    .unwrap()
    .get_str_literal_content()
    .expect("test input must be a str literal");
  let op2 = test_trees
    .last()
    .unwrap()
    .get_str_literal_content()
    .expect("test input must be a str literal");
  out_buffer.push(TokenTree::Literal(Literal::string(&format!(
    "cmp {lhs}, {op2}\nb{cond} {local_label}f\n"
  ))));
  out_buffer.push(TokenTree::Punct(Punct::new(',', Alone)));

  extend_concat_as_lines(&mut out_buffer, body_group.stream());
  // the above fn always leaves a trailing comma, no need for a secondary check.
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
