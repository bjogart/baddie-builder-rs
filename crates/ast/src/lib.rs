mod ast;

use syntax::BbLang;

pub use crate::ast::*;

type SyntaxNode = rowan::SyntaxNode<BbLang>;
type SyntaxToken = rowan::SyntaxToken<BbLang>;
