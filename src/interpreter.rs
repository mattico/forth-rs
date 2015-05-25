
use types::{Int, ForthCell};
use error::{ForthResult, ForthError};
use dictionary::{Dictionary, DictionaryExt, Entry};
use statement::{Statement, StatementExt};
use builtins;

pub type Stack = Vec<Int>;

pub struct Interpreter {
    pub dictionary: Dictionary,
    pub last_result: Option<Stack>,
}

impl Interpreter {
    pub fn new() -> Interpreter {
        let mut dict = Dictionary::new();

        builtins::insert_builtins(&mut dict);

        Interpreter {
            dictionary: dict,
            last_result: None,
        }
    }

    pub fn exec(&mut self, statement: &str) -> ForthResult<()> {
        let mut words = statement.split(|c: char| c.is_whitespace());
        let mut stmt = Statement::new();
        while let Some(word) = words.next() { match word {
            "(" => while let Some(w) = words.next() { if w == ")" { break; } },
            "\\" => while let Some(w) = words.next() { if w == "\n" { break; } },
            ":" => {
                let mut comp = Statement::new();
                let name = match words.next() {
                    Some(w) => w,
                    None => return Err(ForthError::WordNameNotFound),
                };
                while let Some(w) = words.next() {
                    if w == ";" {
                        self.dictionary.insert_entry(Entry::from_statement(name, comp));
                        return Ok(());
                    }
                    match w.parse::<i32>() {
                        Ok(n) => comp.push_back(ForthCell::Number(n)),
                        Err(_) => comp.push_back(ForthCell::Word(self.dictionary.get(w).unwrap().clone())),
                    }
                }
                return Err(ForthError::UnterminatedWordDefinition);
            },
            _ => match word.parse::<i32>() {
                Ok(n) => {
                    stmt.push_back(ForthCell::Number(n));
                },
                Err(_) => {
                    if let Some(elem) = self.dictionary.get(word) {
                        stmt.push_back(ForthCell::Word(elem.clone()));
                    } else {
                        return Err(ForthError::WordNotFound)
                    }
                },
            },

        }}

        match stmt.run(self) {
            Err(e) => {
                self.last_result = None;
                Err(e)
            },
            Ok(s) => {
                self.last_result = Some(s);
                Ok(())
            }
        }
    }
}