mod ast_impl;

use syntax::BbLang;

pub use crate::ast_impl::*;

type SyntaxNode = rowan::SyntaxNode<BbLang>;
type SyntaxToken = rowan::SyntaxToken<BbLang>;
