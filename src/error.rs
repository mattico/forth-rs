
#[derive(Debug, Clone)]
pub enum ForthError {
    EmptyStack,
    UnterminatedComment,
    WordNotFound,
    UnterminatedWordDefinition,
    WordNameNotFound,
    InvalidCharacter,
}

pub type ForthResult<T> = ::std::result::Result<T, ForthError>;