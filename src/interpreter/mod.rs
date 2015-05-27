
use std::default::Default;

use types::{Int, ForthCell};
use error::{ForthResult, ForthError};
use dictionary::Dictionary;
use statement::{Statement, StatementExt};

mod builtins;

pub type Stack = Vec<Int>;

#[derive(Default, Debug)]
pub struct Interpreter {
    pub dictionary: Dictionary,
    pub parameter_stack: Stack,
    pub return_stack: Stack,
    pub statement: Statement,
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

    pub fn parse(&mut self, string: &str) -> ForthResult<()> {
        self.statement = try!(Statement::parse(self, string));
        self.statement_index = 0;
        self.parameter_stack.clear();
        self.return_stack.clear();
        Ok(())
    }

    fn exec_current(&mut self) -> ForthResult<()> {
        match self.get_current().unwrap().clone() {
            ForthCell::Number(n) => { self.parameter_stack.push(n) },
            ForthCell::Word(ref w) => { try!(w.code.run(self)) }
        }
        Ok(())
    }

    fn get_current(&mut self) -> Option<&ForthCell> {
        self.statement.get(self.statement_index)
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

        self.exec_current()
    }

    pub fn prev(&mut self) -> ForthResult<()> {
        if self.has_prev() {
            self.statement_index -= 1;
        } else {
            return Err(ForthError::InvalidJump("No previous statement".to_string()));
        }

        self.exec_current()
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
        try!(self.exec_current());
        while self.has_next() {
            try!(self.next());
        }
        Ok(())
    }

    pub fn exec(&mut self, string: &str) -> ForthResult<()> {
        try!(self.parse(string));
        self.run()

        // let mut stmt = Statement::new();
        // while let Some(word) = words.next() { match word {
        //     "(" => while let Some(w) = words.next() { if w == ")" { break; } },
        //     "\\" => while let Some(w) = words.next() { if w == "\n" { break; } },
        //     ":" => {
        //         let mut comp = Statement::new();
        //         let name = match words.next() {
        //             Some(w) => w,
        //             None => return Err(ForthError::WordNameNotFound),
        //         };
        //         while let Some(w) = words.next() {
        //             if w == ";" {
        //                 self.dictionary.insert_entry(Entry::from_statement(name, comp));
        //                 return Ok(());
        //             }
        //             match w.parse::<i32>() {
        //                 Ok(n) => comp.push_back(ForthCell::Number(n)),
        //                 Err(_) => comp.push_back(ForthCell::Word(.unwrap().clone())),
        //             }
        //         }
        //         return Err(ForthError::UnterminatedWordDefinition);
        //     },
        //     _ => match word.parse::<i32>() {
        //         Ok(n) => {
        //             stmt.push_back(ForthCell::Number(n));
        //         },
        //         Err(_) => {
        //             if let Some(elem) = self.dictionary.get(word) {
        //                 stmt.push_back(ForthCell::Word(elem.clone()));
        //             } else {
        //                 return Err(ForthError::WordNotFound)
        //             }
        //         },
        //     },

        // }}

        // match stmt.run(self) {
        //     Err(e) => {
        //         self.last_result = None;
        //         Err(e)
        //     },
        //     Ok(s) => {
        //         self.last_result = Some(s);
        //         Ok(())
        //     }
        // }
    }
}