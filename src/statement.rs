use std::collections::VecDeque;

use interpreter::{Interpreter, Stack};
use error::ForthResult;
use types::ForthCell;

pub type Statement = VecDeque<ForthCell>;
pub trait StatementExt {
	fn run(&self, &mut Interpreter) -> ForthResult<Stack>;
}

impl StatementExt for Statement {
	fn run(&self, interp: &mut Interpreter) -> ForthResult<Stack> {
		let mut stack = Stack::new();
		for word in self {
			match *word {
				ForthCell::Number(n) => {
					stack.push(n);
				},
				ForthCell::Word(ref w) => {
					let code = w.code.clone();
					try!(code.run(interp, &mut stack));
				},
			};
		};
		Ok(stack)
	}
}