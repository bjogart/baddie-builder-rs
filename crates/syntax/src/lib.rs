use rowan::Language;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BbLang;

pub struct Token;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u16)]
pub enum SyntaxKind {
    DUMMY, // TODO remove variant
    Text,
}

impl Language for BbLang {
    type Kind = SyntaxKind;

    fn kind_from_raw(raw: rowan::SyntaxKind) -> Self::Kind {
        todo!()
    }

    fn kind_to_raw(kind: Self::Kind) -> rowan::SyntaxKind {
        todo!()
    }
}
