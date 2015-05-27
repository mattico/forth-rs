use interpreter::{Interpreter};
use error::ForthResult;
use statement::Statement;
use types::ForthCell;

pub type NativeFn = Box<Fn(&mut Interpreter) -> ForthResult<()>>;

pub enum ForthWord {
    Native(NativeFn),
    Forth(Statement),
}

impl ForthWord {
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