#![allow(dead_code)]

use super::*;

const NOT_ENOUGH_INPUT: &str = "Not enough input";
const ONE_STR_ONLY: &str = "Provide one string literal only.";

/// Generates a unique "local" label string.
pub fn next_local_label() -> String {
  static NEXT: AtomicU64 = AtomicU64::new(0);
  format!(".L_bracer_local_label_{}", NEXT.fetch_add(1, Ordering::Relaxed))
}

/// Gets out the `Group`, if any.
pub fn get_group(tree: TokenTree) -> Option<Group> {
  match tree {
    TokenTree::Group(g) => Some(g),
    _ => None,
  }
}

/// Gets out the `Ident`, if any.
pub fn get_ident(tree: TokenTree) -> Option<Ident> {
  match tree {
    TokenTree::Ident(i) => Some(i),
    _ => None,
  }
}

/// Gets out the `Punct`, if any.
pub fn get_punct(tree: TokenTree) -> Option<Punct> {
  match tree {
    TokenTree::Punct(p) => Some(p),
    _ => None,
  }
}

/// Gets out the `Literal`, if any.
pub fn get_literal(tree: TokenTree) -> Option<Literal> {
  match tree {
    TokenTree::Literal(l) => Some(l),
    _ => None,
  }
}

/// Gets out the `bool`, if any.
pub fn get_bool(tree: &TokenTree) -> Option<bool> {
  match tree {
    TokenTree::Ident(i) => match i.to_string().as_str() {
      "true" => Some(true),
      "false" => Some(false),
      _ => None,
    },
    _ => None,
  }
}

/// Gets the content inside a string literal, if it is one.
pub fn get_str_literal_content(tree: &TokenTree) -> Option<String> {
  match tree {
    TokenTree::Literal(l) => {
      let mut string = format!("{l}");
      if string.starts_with('"') && string.ends_with('"') {
        string.pop();
        string.remove(0);
        Some(string)
      } else {
        None
      }
    }
    _ => None,
  }
}

pub fn one_str_literal_or_panic(token_stream: TokenStream) -> String {
  let mut stream_iter = token_stream.into_iter();
  let literal =
    get_str_literal_content(&stream_iter.next().expect(NOT_ENOUGH_INPUT))
      .expect(ONE_STR_ONLY);
  assert!(stream_iter.next().is_none(), "{ONE_STR_ONLY}");
  literal
}

#[allow(clippy::enum_variant_names)]
pub enum EzTokenTree {
  EzGroup(Delimiter, Vec<EzTokenTree>),
  EzId(String, Span),
  EzPu(char, Spacing),
  EzLi(String),
}
impl EzTokenTree {
  pub fn get_literal(&self) -> Option<&str> {
    match self {
      Self::EzLi(s) => Some(s.as_str()),
      _ => None,
    }
  }
}
impl From<TokenTree> for EzTokenTree {
  fn from(value: TokenTree) -> Self {
    match value {
      TokenTree::Group(g) => EzTokenTree::EzGroup(
        g.delimiter(),
        g.stream().into_iter().map(EzTokenTree::from).collect(),
      ),
      TokenTree::Ident(i) => EzTokenTree::EzId(i.to_string(), i.span()),
      TokenTree::Punct(p) => EzTokenTree::EzPu(p.as_char(), p.spacing()),
      TokenTree::Literal(l) => EzTokenTree::EzLi(l.to_string()),
    }
  }
}
impl From<EzTokenTree> for TokenTree {
  fn from(value: EzTokenTree) -> Self {
    match value {
      EzTokenTree::EzGroup(delimiter, trees) => TokenTree::Group(Group::new(
        delimiter,
        TokenStream::from_iter(trees.into_iter().map(TokenTree::from)),
      )),
      EzTokenTree::EzId(i, s) => TokenTree::Ident(Ident::new(&i, s)),
      EzTokenTree::EzPu(ch, spacing) => {
        TokenTree::Punct(Punct::new(ch, spacing))
      }
      EzTokenTree::EzLi(l) => {
        TokenTree::Literal(Literal::from_str(&l).unwrap())
      }
    }
  }
}
