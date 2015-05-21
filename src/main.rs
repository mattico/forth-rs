#![feature(convert, collections, unicode)]
#![allow(unused_variables)]

extern crate rustc_unicode;

use std::collections::{HashMap, VecDeque};
use std::hash::{Hash, Hasher};
use std::io;
use std::io::prelude::*;
use std::rc::Rc;
use std::fmt;
use std::str::FromStr;
use rustc_unicode::str::UnicodeStr;

type Int = i32;
type Double = f64;

type Dictionary = HashMap<String, Rc<Entry>>;
type Statement = VecDeque<Op>;
type Stack = Vec<Int>;
type NativeFn = Box<Fn(&mut Interpreter, &mut Stack)>;

#[derive(Clone, Debug)]
enum Op {
	Number(Int),
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

	fn from_statement(name: &str, stmt: Statement) -> Entry {
		Entry {
			name: name.to_string(),
			code: Rc::new(Code::Forth(stmt)),
		}
	}
}

impl Code {
	fn run(&self, interp: &mut Interpreter, stack: &mut Stack) {
		match self {
			&Code::Native(ref f) => {
				f(interp, stack);
			},
			&Code::Forth(ref statement) => {
				for ref op in statement {
					match *op {
						&Op::Number(ref n) => { stack.push(*n) }
						&Op::Word(ref w) => {
							(*w).code.run(interp, stack);
						},
					}
				}
			},
		}
	}
}

trait StatementExt {
	fn run(&self, &mut Interpreter);
}

impl StatementExt for Statement {
	fn run(&self, interp: &mut Interpreter) {
		match interp.mode {
			InterpreterMode::Normal => {
				let mut stack = Stack::new();
				for word in self {
					match word {
						&Op::Number(n) => {
							stack.push(n);
						},
						&Op::Word(ref w) => {
							let code = interp.dictionary[w.name.as_str()].code.clone();
							code.run(interp, &mut stack);
						},
					};
				};
			},
			InterpreterMode::Compiler => {
				let mut word = Statement::new();

			},
		}
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

#[derive(PartialEq, Eq)]
enum InterpreterMode {
	Normal,
	Compiler,
}

struct Interpreter {
    dictionary: Dictionary,
    mode: InterpreterMode,
}

macro_rules! binary_entry {
	($name:expr, $o:expr) => {
		Entry::new($name,
				Code::Native(Box::new(|interp: &mut Interpreter, stack: &mut Stack| {
					let b = stack.pop().unwrap();
					let a = stack.pop().unwrap();
					stack.push($o(a, b));
				})),
			)
	}
}

macro_rules! unary_entry {
	($name:expr, $o:expr) => {
		Entry::new($name,
				Code::Native(Box::new(|interp: &mut Interpreter, stack: &mut Stack| {
					let a = stack.pop().unwrap();
					stack.push($o(a));
				})),
			)
	}
}

macro_rules! nonary_entry {
	($name:expr, $o:expr) => {
		Entry::new($name,
				Code::Native(Box::new(|interp: &mut Interpreter, stack: &mut Stack| {
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
				Code::Native(Box::new(|interp: &mut Interpreter, stack: &mut Stack| {
					let y = stack.pop().unwrap();
					let x = stack.pop().unwrap();
					stack.push(if x < y { 1 } else { 0 });
				})),
			));

		dict.insert_entry(Entry::new(">",
				Code::Native(Box::new(|interp: &mut Interpreter, stack: &mut Stack| {
					let y = stack.pop().unwrap();
					let x = stack.pop().unwrap();
					stack.push(if x > y { 1 } else { 0 });
				})),
			));

		dict.insert_entry(Entry::new("=",
				Code::Native(Box::new(|interp: &mut Interpreter, stack: &mut Stack| {
					let y = stack.pop().unwrap();
					let x = stack.pop().unwrap();
					stack.push(if x == y { 1 } else { 0 });
				})),
			));

		dict.insert_entry(Entry::new("0<",
				Code::Native(Box::new(|interp: &mut Interpreter, stack: &mut Stack| {
					let x = stack.pop().unwrap();
					stack.push(if x < 0 { 1 } else { 0 });
				})),
			));

		dict.insert_entry(Entry::new("0=",
				Code::Native(Box::new(|interp: &mut Interpreter, stack: &mut Stack| {
					let x = stack.pop().unwrap();
					stack.push(if x == 0 { 1 } else { 0 });
				})),
			));

		dict.insert_entry(Entry::new("0>",
				Code::Native(Box::new(|interp: &mut Interpreter, stack: &mut Stack| {
					let x = stack.pop().unwrap();
					stack.push(if x > 0 { 1 } else { 0 });
				})),
			));

		dict.insert_entry(Entry::new("1+",
				Code::Native(Box::new(|interp: &mut Interpreter, stack: &mut Stack| {
					let x = stack.pop().unwrap();
					stack.push(x + 1);
				})),
			));

		dict.insert_entry(Entry::new("1-",
				Code::Native(Box::new(|interp: &mut Interpreter, stack: &mut Stack| {
					let x = stack.pop().unwrap();
					stack.push(x - 1);
				})),
			));

		dict.insert_entry(Entry::new("2+",
				Code::Native(Box::new(|interp: &mut Interpreter, stack: &mut Stack| {
					let x = stack.pop().unwrap();
					stack.push(x + 2);
				})),
			));

		dict.insert_entry(Entry::new("2-",
				Code::Native(Box::new(|interp: &mut Interpreter, stack: &mut Stack| {
					let x = stack.pop().unwrap();
					stack.push(x - 2);
				})),
			));

		dict.insert_entry(Entry::new("2/",
				Code::Native(Box::new(|interp: &mut Interpreter, stack: &mut Stack| {
					let x = stack.pop().unwrap();
					stack.push(x >> 1); // Per fst83 standard
				})),
			));

		dict.insert_entry(Entry::new("dup",
				Code::Native(Box::new(|interp: &mut Interpreter, stack: &mut Stack| {
					let x = stack.pop().unwrap();
					stack.push(x.clone());
					stack.push(x);
				})),
			));

		dict.insert_entry(Entry::new("?dup",
				Code::Native(Box::new(|interp: &mut Interpreter, stack: &mut Stack| {
					let x = stack.pop().unwrap();
					if x != 0 {
						stack.push(x.clone());
						stack.push(x);
					} else {
						stack.push(x);
					}
				})),
			));

		dict.insert_entry(Entry::new("over",
				Code::Native(Box::new(|interp: &mut Interpreter, stack: &mut Stack| {
					let x = stack.pop().unwrap();
					let y = stack.pop().unwrap();
					stack.push(x.clone());
					stack.push(y);
					stack.push(x);
				})),
			));

		dict.insert_entry(Entry::new("swap",
				Code::Native(Box::new(|interp: &mut Interpreter, stack: &mut Stack| {
					let x = stack.pop().unwrap();
					let y = stack.pop().unwrap();
					stack.push(x);
					stack.push(y);
				})),
			));

		dict.insert_entry(Entry::new("rot",
				Code::Native(Box::new(|interp: &mut Interpreter, stack: &mut Stack| {
					let x = stack.pop().unwrap();
					let y = stack.pop().unwrap();
					let z = stack.pop().unwrap();
					stack.push(y);
					stack.push(x);
					stack.push(z);
				})),
			));

		dict.insert_entry(Entry::new("dump",
				Code::Native(Box::new(|interp: &mut Interpreter, stack: &mut Stack| {
					println!("ds =  {:?} ", stack);
				})),
			));

		dict.insert_entry(Entry::new("cr",
				Code::Native(Box::new(|interp: &mut Interpreter, stack: &mut Stack| {
					println!("");
				})),
			));

		dict.insert_entry(Entry::new(".",
				Code::Native(Box::new(|interp: &mut Interpreter, stack: &mut Stack| {
					println!("{}", stack.pop().unwrap());
				})),
			));

		dict.insert_entry(Entry::new(":",
				Code::Native(Box::new(|interp: &mut Interpreter, stack: &mut Stack| {
					if interp.mode != InterpreterMode::Compiler {
						interp.mode = InterpreterMode::Compiler;
					} else {
						panic!("Compilerception!")
					}
				})),
			));

		dict.insert_entry(Entry::new(";",
				Code::Native(Box::new(|interp: &mut Interpreter, stack: &mut Stack| {
					if interp.mode == InterpreterMode::Compiler {
						interp.mode = InterpreterMode::Normal;
					} else {
						panic!("Got ';' when not in Compile mode")
					}
				})),
			));

		let square = {
			let mut s = Statement::new();
			s.push_back(Op::Word(dict.get("dup").unwrap().clone()));
			s.push_back(Op::Word(dict.get("*").unwrap().clone()));
			Code::Forth(s)
		};
		dict.insert_entry(Entry::new("square", square));

		Interpreter {
			dictionary: dict,
			mode: InterpreterMode::Normal,
		}
	}

	fn exec(&mut self, statement: &str) {
		let mut words = statement.split(|c: char| c.is_whitespace());
		let mut stmt = Statement::new();
		while let Some(word) = words.next() { match word {
			"(" => while let Some(w) = words.next() { if w == ")" { break; } },
			"\\" => while let Some(w) = words.next() { if w == "\n" { break; } },
			":" => {
				let mut comp = Statement::new();
				let name = words.next().unwrap();
				while let Some(w) = words.next() {
					if w == ";" { break; }
					match w.parse::<i32>() {
						Ok(n) => comp.push_back(Op::Number(n)),
						Err(_) => comp.push_back(Op::Word(self.dictionary.get(w).unwrap().clone())),
					}
				}
				self.dictionary.insert_entry(Entry::new(name, Code::Forth(comp)));
			},
			_ => match word.parse::<i32>() {
				Ok(n) => {
					stmt.push_back(Op::Number(n));
				},
				Err(e) => {
					if let Some(elem) = self.dictionary.get(word) {
						stmt.push_back(Op::Word(elem.clone()));
					} else {
						println!("Invalid word: {}", word);
						return;
					}
				},
			},

		}}

		stmt.run(self);
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
