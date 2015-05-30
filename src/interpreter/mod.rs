
use std::default::Default;

use types::{Int, ForthCell};
use error::{ForthResult, ForthError};
use dictionary::{Dictionary};

mod builtins;

pub type Stack = Vec<Int>;

#[derive(Default, Debug)]
pub struct Interpreter {
    pub dictionary: Dictionary,
    pub parameter_stack: Stack,
    pub return_stack: Stack,
    pub statement: Vec<String>,
    pub statement_index: usize,
}

// TODO: Think about refactoring this to make it less... meh.
impl Interpreter {
    pub fn new() -> Interpreter {
        let mut dict = Dictionary::new();

        builtins::insert_builtins(&mut dict);

        Interpreter {
            dictionary: dict,
            ..Default::default()
        }
    }

    pub fn parse(&mut self, string: String)
    {
        self.statement = string.split_whitespace().map(|s| s.to_string()).collect();
        self.statement_index = 0;
    }

    pub fn parse_word(&self, word: String) -> ForthResult<ForthCell> {
        if let Some(w) = self.dictionary.get(word.as_str()) {
            return Ok(ForthCell::Word(w.clone()));
        } else if let Ok(n) = word.parse::<Int>() {
            return Ok(ForthCell::Number(n));
        }

        return Err(ForthError::WordNotFound);
    }

    pub fn exec_current(&mut self) -> ForthResult<()> {
        match self.get_current() {
            Some(s) => {
                if let Ok(w) = self.parse_word(s) {
                    match w {
                        ForthCell::Word(w) => return w.code.run(self),
                        ForthCell::Number(n) => self.parameter_stack.push(n),
                    }
                } else {
                    return Err(ForthError::WordNotFound);
                }
            },
            None => return Err(ForthError::InvalidJump("No current statement".to_string())),
        }

        Ok(())
    }

    pub fn get_current(&mut self) -> Option<String> {
        match self.statement.get(self.statement_index) {
            Some(s) => Some(s.to_string()),
            None => None,
        }
    }

    pub fn has_next(&self) -> bool {
        self.statement_index < self.statement.len() - 1
    }

    pub fn has_prev(&self) -> bool {
        self.statement_index > 0
    }

    pub fn next(&mut self) -> ForthResult<()> {
        if self.has_next() {
            self.statement_index += 1;
        } else {
            return Err(ForthError::InvalidJump("No next statement".to_string()));
        }

        Ok(())
    }

    pub fn prev(&mut self) -> ForthResult<()> {
        if self.has_prev() {
            self.statement_index -= 1;
        } else {
            return Err(ForthError::InvalidJump("No previous statement".to_string()));
        }

        Ok(())
    }

    pub fn jump(&mut self, offset: i32) -> ForthResult<()> {
        let target = self.statement_index as isize + offset as isize;
        if target < self.statement.len() as isize && target >= 0 {
            self.statement_index = target as usize;
        } else {
            return Err(ForthError::InvalidJump(format!("Invalid jump offset {} to index {}", offset, target).to_string()));
        }

        Ok(())
    }

    pub fn run(&mut self) -> ForthResult<()> {
        if self.statement.len() == 0 { return Ok(()) }
        try!(self.exec_current());
        while self.has_next() {
            try!(self.next());
            try!(self.exec_current());
        }

        Ok(())
    }

    pub fn exec(&mut self, string: String) -> ForthResult<()> {
        self.parse(string);
        self.run()
    }
}