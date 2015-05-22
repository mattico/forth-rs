
#[derive(Debug, Clone)]
pub enum ForthError {
    EmptyStack,
    UnterminatedComment,
    WordNotFound,
    UnterminatedWordDefinition,
    WordNameNotFound,
}

pub type ForthResult<T> = ::std::result::Result<T, ForthError>;