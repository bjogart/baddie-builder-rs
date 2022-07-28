use ast::Text;
use syntax::Token;

pub struct Error;

pub fn parse(tokens: impl Iterator<Item = Token>) -> (Text, Vec<Error>) {
    todo!()
}
