use interpreter::{Interpreter};
use error::{ForthResult};
use types::{ForthCell, Statement};

pub type NativeFn = Box<Fn(&mut Interpreter) -> ForthResult<()>>;

pub enum ForthWord {
    Native(NativeFn),
    Forth(Statement),
}

impl ForthWord {
    pub fn parse(interp: &Interpreter, string: String) -> ForthResult<Statement> {
        let mut stmt = Statement::new();

        let mut words = string.split_whitespace();
        while let Some(word) = words.next() {
            stmt.push_back(try!(interp.parse_word(word.to_string())));
        }
        Ok(stmt)
    }

    pub fn run(&self, interp: &mut Interpreter) -> ForthResult<()> {
        match *self {
            ForthWord::Native(ref f) => {
                return f(interp);
            },
            // TODO: DRY with interpreter, statement
            ForthWord::Forth(ref statement) => {
                for ref op in statement {
                    match **op {
                        ForthCell::Number(ref n) => { interp.parameter_stack.push(*n) }
                        ForthCell::Word(ref w) => {
                            try!((*w).code.run(interp));
                        },
                    }
                }
            },
        }
        Ok(())
    }
}