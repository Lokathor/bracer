#![allow(dead_code)]

use super::*;

const NOT_ENOUGH_INPUT: &str = "Not enough input";
const ONE_STR_ONLY: &str = "Provide one string literal only.";

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
