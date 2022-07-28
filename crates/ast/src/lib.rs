mod wrappers;

use syntax::BbLang;

pub use crate::wrappers::*;

type SyntaxNode = rowan::SyntaxNode<BbLang>;
type SyntaxToken = rowan::SyntaxToken<BbLang>;
