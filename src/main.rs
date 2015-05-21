#![feature(convert, collections, unicode)]
#![allow(unused_variables)]

extern crate rustc_unicode;

use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::io;
use std::io::prelude::*;
use std::rc::Rc;
use std::fmt;
use rustc_unicode::str::UnicodeStr;

type Dictionary = HashMap<String, Rc<Entry>>;
type Statement = Vec<Op>;
type NativeFn = Box<Fn(&mut Vec<Op>, &mut Interpreter)>;

#[derive(Clone, Debug)]
enum Op {
	Number(i32),
	Word(Rc<Entry>),
}

enum Code {
	Native(NativeFn),
	Forth(Statement),
}

#[derive(Clone)]
struct Entry {
	name: String,
	code: Rc<Code>,
}

impl PartialEq for Entry {
	fn eq(&self, other: &Self) -> bool { self.name == other.name }
}

impl Eq for Entry {}

impl Hash for Entry {
	fn hash<H>(&self, state: &mut H) where H: Hasher {
		self.name.hash(state)
	}
}

impl fmt::Debug for Entry {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
		self.name.fmt(fmt)
	}
}

impl Entry {
	fn new(name: &str, code: Code) -> Entry {
		Entry {
			name: String::from_str(name),
			code: Rc::new(code),
		}
	}
}

impl Code {
	fn run(&self, stack: &mut Statement, interp: &mut Interpreter) {
		match self {
			&Code::Native(ref f) => {
				f(stack, interp);
			},
			&Code::Forth(ref statement) => {
				for ref op in statement {
					match *op {
						&Op::Number(ref n) => { break; }
						&Op::Word(ref w) => {
							(*w).code.run(stack, interp);
						},
					}
				}
			},
		}
	}
}

trait StatementExt {
	fn pop_number(&mut self, &mut Interpreter) -> i32;
	fn push_number(&mut self, i32);
	fn run(&mut self, &mut Interpreter) -> Result<i32, Statement>;
}

impl StatementExt for Statement {
	fn pop_number(&mut self, interp: &mut Interpreter) -> i32 {
		loop {
			match self.pop() {
				Some(Op::Number(n)) => { return n; },
				Some(Op::Word(s)) => s.code.run(self, interp),
				None => panic!("Popped empty statement"),
			}
		}
	}

	fn push_number(&mut self, number: i32) {
		self.push(Op::Number(number));
	}

	fn run(&mut self, interp: &mut Interpreter) -> Result<i32, Statement> {
		while self.len() > 0 {
			match self.pop() {
				Some(Op::Number(n)) => {
					if self.len() == 0 {
						return Ok(n)
					} else {
						self.push(Op::Number(n));
						return Err(self.clone())
					}
				},
				Some(Op::Word(w)) => {
					let code = interp.dictionary[w.name.as_str()].code.clone();
					code.run(self, interp);
				},
				None => unreachable!(),
			};
		}
		Ok(0)
	}
}

trait DictionaryExt {
	fn insert_entry(&mut self, Entry);
}

impl DictionaryExt for Dictionary {
	fn insert_entry(&mut self, entry: Entry) {
		self.insert(entry.name.clone(), Rc::new(entry));
	}
}

struct Interpreter {
    dictionary: Dictionary,
}

macro_rules! binary_entry {
	($name:expr, $o:expr) => {
		Entry::new($name,
				Code::Native(Box::new(|stack: &mut Statement, interp: &mut Interpreter| {
					let b = stack.pop_number(interp);
					let a = stack.pop_number(interp);
					stack.push_number($o(a, b));
				})),
			)
	}
}

macro_rules! unary_entry {
	($name:expr, $o:expr) => {
		Entry::new($name,
				Code::Native(Box::new(|stack: &mut Statement, interp: &mut Interpreter| {
					let a = stack.pop_number(interp);
					stack.push_number($o(a));
				})),
			)
	}
}

macro_rules! nonary_entry {
	($name:expr, $o:expr) => {
		Entry::new($name,
				Code::Native(Box::new(|stack: &mut Statement, interp: &mut Interpreter| {
					Op::Number($o);
				})),
			)
	}
}

impl Interpreter {
	fn new() -> Interpreter {
		let mut dict = Dictionary::new();

		dict.insert_entry(binary_entry!("/", ::std::ops::Div::div));
		dict.insert_entry(binary_entry!("*", ::std::ops::Mul::mul));
		dict.insert_entry(binary_entry!("+", ::std::ops::Add::add));
		dict.insert_entry(binary_entry!("-", ::std::ops::Sub::sub));
		dict.insert_entry(binary_entry!("mod", ::std::ops::Rem::rem));
		dict.insert_entry(binary_entry!("and", ::std::ops::BitAnd::bitand));
		dict.insert_entry(binary_entry!("or", ::std::ops::BitOr::bitor));
		dict.insert_entry(binary_entry!("xor", ::std::ops::BitXor::bitxor));
		dict.insert_entry(binary_entry!("rshift", ::std::ops::Shr::shr));
		dict.insert_entry(binary_entry!("lshift", ::std::ops::Shl::shl));

		dict.insert_entry(unary_entry!("negate", ::std::ops::Neg::neg));
		dict.insert_entry(unary_entry!("not", ::std::ops::Not::not));

		dict.insert_entry(nonary_entry!("bye", ::std::process::exit(0)));

		dict.insert_entry(Entry::new("<",
				Code::Native(Box::new(|stack: &mut Statement, interp: &mut Interpreter| {
					let y = stack.pop_number(interp);
					let x = stack.pop_number(interp);
					stack.push(if x < y { Op::Number(1) } else { Op::Number(0) });
				})),
			));

		dict.insert_entry(Entry::new(">",
				Code::Native(Box::new(|stack: &mut Statement, interp: &mut Interpreter| {
					let y = stack.pop_number(interp);
					let x = stack.pop_number(interp);
					stack.push(if x > y { Op::Number(1) } else { Op::Number(0) });
				})),
			));

		dict.insert_entry(Entry::new("=",
				Code::Native(Box::new(|stack: &mut Statement, interp: &mut Interpreter| {
					let y = stack.pop_number(interp);
					let x = stack.pop_number(interp);
					stack.push(if x == y { Op::Number(1) } else { Op::Number(0) });
				})),
			));

		dict.insert_entry(Entry::new("0<",
				Code::Native(Box::new(|stack: &mut Statement, interp: &mut Interpreter| {
					let x = stack.pop_number(interp);
					stack.push(if x < 0 { Op::Number(1) } else { Op::Number(0) });
				})),
			));

		dict.insert_entry(Entry::new("0=",
				Code::Native(Box::new(|stack: &mut Statement, interp: &mut Interpreter| {
					let x = stack.pop_number(interp);
					stack.push(if x == 0 { Op::Number(1) } else { Op::Number(0) });
				})),
			));

		dict.insert_entry(Entry::new("0>",
				Code::Native(Box::new(|stack: &mut Statement, interp: &mut Interpreter| {
					let x = stack.pop_number(interp);
					stack.push(if x > 0 { Op::Number(1) } else { Op::Number(0) });
				})),
			));

		dict.insert_entry(Entry::new("1+",
				Code::Native(Box::new(|stack: &mut Statement, interp: &mut Interpreter| {
					let x = stack.pop_number(interp);
					stack.push_number(x + 1);
				})),
			));

		dict.insert_entry(Entry::new("1-",
				Code::Native(Box::new(|stack: &mut Statement, interp: &mut Interpreter| {
					let x = stack.pop_number(interp);
					stack.push_number(x - 1);
				})),
			));

		dict.insert_entry(Entry::new("2+",
				Code::Native(Box::new(|stack: &mut Statement, interp: &mut Interpreter| {
					let x = stack.pop_number(interp);
					stack.push_number(x + 2);
				})),
			));

		dict.insert_entry(Entry::new("2-",
				Code::Native(Box::new(|stack: &mut Statement, interp: &mut Interpreter| {
					let x = stack.pop_number(interp);
					stack.push_number(x - 2);
				})),
			));

		dict.insert_entry(Entry::new("2/",
				Code::Native(Box::new(|stack: &mut Statement, interp: &mut Interpreter| {
					let x = stack.pop_number(interp);
					stack.push_number(x >> 1); // Per fst83 standard
				})),
			));

		dict.insert_entry(Entry::new("dup",
				Code::Native(Box::new(|stack: &mut Statement, interp: &mut Interpreter| {
					let x = stack.pop_number(interp);
					stack.push_number(x.clone());
					stack.push_number(x);
				})),
			));

		dict.insert_entry(Entry::new("?dup",
				Code::Native(Box::new(|stack: &mut Statement, interp: &mut Interpreter| {
					let x = stack.pop_number(interp);
					if x != 0 {
						stack.push_number(x.clone());
						stack.push_number(x);
					} else {
						stack.push_number(x);
					}
				})),
			));

		dict.insert_entry(Entry::new("over",
				Code::Native(Box::new(|stack: &mut Statement, interp: &mut Interpreter| {
					let x = stack.pop_number(interp);
					let y = stack.pop_number(interp);
					stack.push_number(x.clone());
					stack.push_number(y);
					stack.push_number(x);
				})),
			));

		dict.insert_entry(Entry::new("rot",
				Code::Native(Box::new(|stack: &mut Statement, interp: &mut Interpreter| {
					let x = stack.pop_number(interp);
					let y = stack.pop_number(interp);
					let z = stack.pop_number(interp);
					stack.push_number(y);
					stack.push_number(x);
					stack.push_number(z);
				})),
			));

		dict.insert_entry(Entry::new("dump",
				Code::Native(Box::new(|stack: &mut Statement, interp: &mut Interpreter| {
					let x = stack.pop().unwrap();
					print!("{:?} ", x);
					stack.push(x);
				})),
			));

		dict.insert_entry(Entry::new("cr",
				Code::Native(Box::new(|stack: &mut Statement, interp: &mut Interpreter| {
					println!("");
				})),
			));

		dict.insert_entry(Entry::new(".",
				Code::Native(Box::new(|stack: &mut Statement, interp: &mut Interpreter| {
					println!("{:?}", stack.pop_number(interp));
				})),
			));

		Interpreter {
			dictionary: dict,
		}
	}

	fn lex(&mut self, statement: &str) -> Statement {
		let words = statement.split(' ');
		let mut ret = Statement::new();
		for word in words {
			if let Some(n) = i32::from_str_radix(word, 10).ok() {
				ret.push(Op::Number(n));
			} else if (*word).is_whitespace() {
				return ret;
			} else {
				ret.push(Op::Word(self.dictionary[word.to_lowercase().as_str()].clone()));
			}
		};
		ret
	}

	fn exec(&mut self, statement: &str) -> Result<i32, Statement> {
		let mut statement = self.lex(statement);
		return statement.run(self);
	}
}

fn main() {
	println!("forth-rs version 0.1.0");
	println!("Type 'bye' to exit");
    let mut interp = Interpreter::new();
    let mut stdin = io::stdin();
    loop {
    	print!("Forth> ");
    	io::stdout().flush().ok().expect("Could not flush stdout");
    	let mut line = String::new();
    	stdin.read_line(&mut line).ok().expect("Unable to read from stdin");
    	let result = interp.exec(line.trim());
    }
}
