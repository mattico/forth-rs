use std::collections::VecDeque;

use interpreter::{Interpreter};
use error::{ForthResult, ForthError};
use types::{ForthCell, Int};

// TODO: ForthCell -> Result<ForthCell, String>
//       for lazy evaluation of possibly incorrect stuff ?
pub type Statement = VecDeque<ForthCell>;
pub trait StatementExt {
    fn parse(&Interpreter, &str) -> ForthResult<Statement>;
}

impl StatementExt for Statement {
    fn parse(interp: &Interpreter, string: &str) -> ForthResult<Statement> {
        let mut stmt = Statement::new();

        let mut words = string.split_whitespace();
        while let Some(word) = words.next() {
            if let Some(w) = interp.dictionary.get(word) {
                stmt.push_back(ForthCell::Word(w.clone()));
            } else if let Ok(n) = word.parse::<Int>() {
                stmt.push_back(ForthCell::Number(n));
            } else if word == "(" {
                while let Some(w) = words.next().clone() {
                    if w.rfind(")").unwrap_or(w.len()) == w.len() - 1 { break; }
                }
            } else {
                return Err(ForthError::WordNotFound);
            }
        }
        Ok(stmt)
    }
}