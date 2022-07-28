use crate::SyntaxNode;
use crate::SyntaxToken;
use rowan::ast::AstNode;
use rowan::Language;
use rowan::NodeOrToken;
use syntax::BbLang;
use syntax::SyntaxKind;
#[derive(Debug, Clone)]
pub struct Text(SyntaxNode);
impl AstNode for Text {
    type Language = BbLang;
    fn can_cast(kind: <Self::Language as Language>::Kind) -> bool {
        matches!(kind, SyntaxKind::Text)
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self(syntax))
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.0
    }
}
impl Text {
    pub fn dummy(&self) -> Option<SyntaxToken> {
        self.0.children_with_tokens().find_map(|c| match c {
            NodeOrToken::Token(c) if matches!(c.kind(), SyntaxKind::DUMMY) => Some(c),
            _ => None,
        })
    }
}
