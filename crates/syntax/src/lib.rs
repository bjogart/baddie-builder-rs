use rowan::Language;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BbLang;

pub struct Token;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u16)]
pub enum SyntaxKind {
    DUMMY, // TODO development variant, remove at 1.0
    Text,
}

impl Language for BbLang {
    type Kind = SyntaxKind;

    fn kind_from_raw(_raw: rowan::SyntaxKind) -> Self::Kind {
        todo!()
    }

    fn kind_to_raw(_kind: Self::Kind) -> rowan::SyntaxKind {
        todo!()
    }
}
