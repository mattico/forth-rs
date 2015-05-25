use interpreter::{Interpreter, Stack};
use error::ForthResult;
use statement::Statement;
use types::ForthCell;

pub type NativeFn = Box<Fn(&mut Interpreter, &mut Stack) -> ForthResult<()>>;

pub enum ForthWord {
	Native(NativeFn),
	Forth(Statement),
}

impl ForthWord {
	pub fn run(&self, interp: &mut Interpreter, stack: &mut Stack) -> ForthResult<()> {
		match *self {
			ForthWord::Native(ref f) => {
				return f(interp, stack);
			},
			ForthWord::Forth(ref statement) => {
				for ref op in statement {
					match *op {
						&ForthCell::Number(ref n) => { stack.push(*n) }
						&ForthCell::Word(ref w) => {
							try!((*w).code.run(interp, stack));
						},
					}
				}
			},
		}
		Ok(())
	}
}