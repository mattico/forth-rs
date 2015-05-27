#[derive(Debug, Clone)]
pub enum ForthError {
    EmptyStack,
    UnterminatedComment,
    WordNotFound,
    UnterminatedWordDefinition,
    WordNameNotFound,
    InvalidCharacter,
    InvalidJump(String),
    ExpectedNumber,
    SemicolonOutsideOfWordDefinition,
}

pub type ForthResult<T> = ::std::result::Result<T, ForthError>;