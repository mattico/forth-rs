#![feature(convert, collections, unicode)]

use std::collections::{HashMap, VecDeque};
use std::hash::{Hash, Hasher};
use std::io;
use std::io::prelude::*;
use std::rc::Rc;
use std::fmt;

extern crate rustc_unicode;
use rustc_unicode::str::UnicodeStr;

type Int = i32;
type Double = f64;

#[derive(Debug, Clone)]
enum ForthError {
    EmptyStack,
    UnterminatedComment,
    WordNotFound,
    UnterminatedWordDefinition,
    WordNameNotFound,
}

type ForthResult<T> = std::result::Result<T, ForthError>;

type Dictionary = HashMap<String, Rc<Entry>>;
type Statement = VecDeque<Op>;
type Stack = Vec<Int>;
type NativeFn = Box<Fn(&mut Interpreter, &mut Stack) -> ForthResult<()>>;

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
	fn run(&self, interp: &mut Interpreter, stack: &mut Stack) -> ForthResult<()> {
		match self {
			&Code::Native(ref f) => {
				return f(interp, stack);
			},
			&Code::Forth(ref statement) => {
				for ref op in statement {
					match *op {
						&Op::Number(ref n) => { stack.push(*n) }
						&Op::Word(ref w) => {
							try!((*w).code.run(interp, stack));
						},
					}
				}
			},
		}
		Ok(())
	}
}

trait StatementExt {
	fn run(&self, &mut Interpreter) -> ForthResult<Stack>;
}

impl StatementExt for Statement {
	fn run(&self, interp: &mut Interpreter) -> ForthResult<Stack> {
		let mut stack = Stack::new();
		for word in self {
			match word {
				&Op::Number(n) => {
					stack.push(n);
				},
				&Op::Word(ref w) => {
					let code = interp.dictionary[w.name.as_str()].code.clone();
					try!(code.run(interp, &mut stack));
				},
			};
		};
		Ok(stack)
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
    last_result: Option<Stack>,
}

macro_rules! try_stack {
	($x:expr) => {
		match $x {
			Some(i) => i,
			None => return Err(ForthError::EmptyStack),
		}
	}
}

macro_rules! binary_entry {
	($name:expr, $o:expr) => {
		Entry::new($name,
				Code::Native(Box::new(|_: &mut Interpreter, stack: &mut Stack| -> ForthResult<()> {
					let b = try_stack!(stack.pop());
					let a = try_stack!(stack.pop());
					stack.push($o(a, b));
					Ok(())
				})),
			)
	}
}

macro_rules! unary_entry {
	($name:expr, $o:expr) => {
		Entry::new($name,
				Code::Native(Box::new(|_: &mut Interpreter, stack: &mut Stack| -> ForthResult<()> {
					let a = try_stack!(stack.pop());
					stack.push($o(a));
					Ok(())
				})),
			)
	}
}

macro_rules! nonary_entry {
	($name:expr, $o:expr) => {
		Entry::new($name,
				Code::Native(Box::new(|_: &mut Interpreter, _: &mut Stack| -> ForthResult<()> {
					Op::Number($o);
					Ok(())
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

		dict.insert_entry(Entry::new("$",
				Code::Native(Box::new(|interp: &mut Interpreter, stack: &mut Stack| -> ForthResult<()> {
					let x = try_stack!(interp.last_result.clone());
					for v in x {
						stack.push(v);
					}
					Ok(())
				})),
			));

		dict.insert_entry(Entry::new("<",
				Code::Native(Box::new(|_: &mut Interpreter, stack: &mut Stack| -> ForthResult<()> {
					let y = try_stack!(stack.pop());
					let x = try_stack!(stack.pop());
					stack.push(if x < y { 1 } else { 0 });
					Ok(())
				})),
			));

		dict.insert_entry(Entry::new(">",
				Code::Native(Box::new(|_: &mut Interpreter, stack: &mut Stack| -> ForthResult<()> {
					let y = try_stack!(stack.pop());
					let x = try_stack!(stack.pop());
					stack.push(if x > y { 1 } else { 0 });
					Ok(())
				})),
			));

		dict.insert_entry(Entry::new("=",
				Code::Native(Box::new(|_: &mut Interpreter, stack: &mut Stack| -> ForthResult<()> {
					let y = try_stack!(stack.pop());
					let x = try_stack!(stack.pop());
					stack.push(if x == y { 1 } else { 0 });
					Ok(())
				})),
			));

		dict.insert_entry(Entry::new("0<",
				Code::Native(Box::new(|_: &mut Interpreter, stack: &mut Stack| -> ForthResult<()> {
					let x = try_stack!(stack.pop());
					stack.push(if x < 0 { 1 } else { 0 });
					Ok(())
				})),
			));

		dict.insert_entry(Entry::new("0=",
				Code::Native(Box::new(|_: &mut Interpreter, stack: &mut Stack| -> ForthResult<()> {
					let x = try_stack!(stack.pop());
					stack.push(if x == 0 { 1 } else { 0 });
					Ok(())
				})),
			));

		dict.insert_entry(Entry::new("0>",
				Code::Native(Box::new(|_: &mut Interpreter, stack: &mut Stack| -> ForthResult<()> {
					let x = try_stack!(stack.pop());
					stack.push(if x > 0 { 1 } else { 0 });
					Ok(())
				})),
			));

		dict.insert_entry(Entry::new("1+",
				Code::Native(Box::new(|_: &mut Interpreter, stack: &mut Stack| -> ForthResult<()> {
					let x = try_stack!(stack.pop());
					stack.push(x + 1);
					Ok(())
				})),
			));

		dict.insert_entry(Entry::new("1-",
				Code::Native(Box::new(|_: &mut Interpreter, stack: &mut Stack| -> ForthResult<()> {
					let x = try_stack!(stack.pop());
					stack.push(x - 1);
					Ok(())
				})),
			));

		dict.insert_entry(Entry::new("2+",
				Code::Native(Box::new(|_: &mut Interpreter, stack: &mut Stack| -> ForthResult<()> {
					let x = try_stack!(stack.pop());
					stack.push(x + 2);
					Ok(())
				})),
			));

		dict.insert_entry(Entry::new("2-",
				Code::Native(Box::new(|_: &mut Interpreter, stack: &mut Stack| -> ForthResult<()> {
					let x = try_stack!(stack.pop());
					stack.push(x - 2);
					Ok(())
				})),
			));

		dict.insert_entry(Entry::new("2/",
				Code::Native(Box::new(|_: &mut Interpreter, stack: &mut Stack| -> ForthResult<()> {
					let x = try_stack!(stack.pop());
					stack.push(x >> 1); // Per fst83 standard
					Ok(())
				})),
			));

		dict.insert_entry(Entry::new("dup",
				Code::Native(Box::new(|_: &mut Interpreter, stack: &mut Stack| -> ForthResult<()> {
					let x = try_stack!(stack.pop());
					stack.push(x.clone());
					stack.push(x);
					Ok(())
				})),
			));

		dict.insert_entry(Entry::new("?dup",
				Code::Native(Box::new(|_: &mut Interpreter, stack: &mut Stack| -> ForthResult<()> {
					let x = try_stack!(stack.pop());
					if x != 0 {
						stack.push(x.clone());
						stack.push(x);
					} else {
						stack.push(x);
					}
					Ok(())
				})),
			));

		dict.insert_entry(Entry::new("over",
				Code::Native(Box::new(|_: &mut Interpreter, stack: &mut Stack| -> ForthResult<()> {
					let x = try_stack!(stack.pop());
					let y = try_stack!(stack.pop());
					stack.push(x.clone());
					stack.push(y);
					stack.push(x);
					Ok(())
				})),
			));

		dict.insert_entry(Entry::new("swap",
				Code::Native(Box::new(|_: &mut Interpreter, stack: &mut Stack| -> ForthResult<()> {
					let x = try_stack!(stack.pop());
					let y = try_stack!(stack.pop());
					stack.push(x);
					stack.push(y);
					Ok(())
				})),
			));

		dict.insert_entry(Entry::new("rot",
				Code::Native(Box::new(|_: &mut Interpreter, stack: &mut Stack| -> ForthResult<()> {
					let x = try_stack!(stack.pop());
					let y = try_stack!(stack.pop());
					let z = try_stack!(stack.pop());
					stack.push(y);
					stack.push(x);
					stack.push(z);
					Ok(())
				})),
			));

		dict.insert_entry(Entry::new("dump",
				Code::Native(Box::new(|_: &mut Interpreter, stack: &mut Stack| -> ForthResult<()> {
					println!("ds =  {:?} ", stack);
					Ok(())
				})),
			));

		dict.insert_entry(Entry::new("cr",
				Code::Native(Box::new(|_: &mut Interpreter, _: &mut Stack| -> ForthResult<()> {
					println!("");
					Ok(())
				})),
			));

		dict.insert_entry(Entry::new(".",
				Code::Native(Box::new(|_: &mut Interpreter, stack: &mut Stack| -> ForthResult<()> {
					println!("{}", try_stack!(stack.pop()));
					Ok(())
				})),
			));

		let square = {
			let mut s = Statement::new();
			s.push_back(Op::Word(dict.get("dup").unwrap().clone()));
			s.push_back(Op::Word(dict.get("*").unwrap().clone()));
			s
		};
		dict.insert_entry(Entry::from_statement("square", square));

		Interpreter {
			dictionary: dict,
			last_result: None,
		}
	}

	fn exec(&mut self, statement: &str) -> ForthResult<()> {
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
						Ok(n) => comp.push_back(Op::Number(n)),
						Err(_) => comp.push_back(Op::Word(self.dictionary.get(w).unwrap().clone())),
					}
				}
				return Err(ForthError::UnterminatedWordDefinition);
			},
			_ => match word.parse::<i32>() {
				Ok(n) => {
					stmt.push_back(Op::Number(n));
				},
				Err(_) => {
					if let Some(elem) = self.dictionary.get(word) {
						stmt.push_back(Op::Word(elem.clone()));
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
    	match interp.exec(line.trim()) {
    		Err(e) => println!("{:?}", e),
    		Ok(_) => {},
    	}
    }
}
