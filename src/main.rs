#![feature(float_from_str_radix, convert, collections)]

use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::io;
use std::io::prelude::*;
use std::rc::Rc;

type Dictionary = HashMap<String, Rc<Entry>>;
type Statement = Vec<Op>;
type NativeFn = Box<Fn(&mut Vec<Op>, &mut Interpreter)>;

#[derive(Clone)]
enum Op {
	Number(f64),
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

impl Entry {
	fn new(name: String, code: Code) -> Entry {
		Entry {
			name: name,
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
	fn pop_number(&mut self, &mut Interpreter) -> f64;
	fn run(&mut self, &mut Interpreter) -> f64;
}

impl StatementExt for Statement {
	fn pop_number(&mut self, interp: &mut Interpreter) -> f64 {
		loop {
			match self.pop() {
				Some(Op::Number(n)) => { return n; },
				Some(Op::Word(s)) => s.code.run(self, interp),
				None => { panic!("Popped empty statement"); },
			}
		}
	}
	fn run(&mut self, interp: &mut Interpreter) -> f64 {
		while self.len() > 0 {
			match self.pop() {
				Some(Op::Number(n)) => { return n },
				Some(Op::Word(w)) => {
					let code = interp.dictionary[w.name.as_str()].code.clone();
					code.run(self, interp);
				},
				None => { panic!("Empty Statement") },
			};
		}
		unreachable!();
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
		Entry::new(
				String::from_str($name),
				Code::Native(Box::new(|stack: &mut Statement, interp: &mut Interpreter| {
					let b = stack.pop_number(interp);
					let a = stack.pop_number(interp);
					stack.push(Op::Number($o(a, b)));
				})),
			)
	}
}

macro_rules! unary_entry {
	($name:expr, $o:expr) => {
		Entry::new(
				String::from_str($name),
				Code::Native(Box::new(|stack: &mut Statement, interp: &mut Interpreter| {
					let b = stack.pop_number(interp);
					stack.push(Op::Number($o(a)));
				})),
			)
	}
}

macro_rules! nonary_entry {
	($name:expr, $o:expr) => {
		Entry::new(
				String::from_str($name),
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
		dict.insert_entry(binary_entry!("%", ::std::ops::Rem::rem));

		dict.insert_entry(nonary_entry!("QUIT", ::std::process::exit(0)));
		dict.insert_entry(nonary_entry!("bye", ::std::process::exit(0)));


		Interpreter {
			dictionary: dict,
		}
	}

	fn lex(&mut self, statement: String) -> Statement {
		let words = statement.split(' ');
		let mut ret = Statement::new();
		for word in words {
			if let Some(n) = f64::from_str_radix(word, 10).ok() {
				ret.push(Op::Number(n));
			} else {
				ret.push(Op::Word(self.dictionary[word].clone()));
			}
		};
		ret
	}

	fn exec(&mut self, statement: String) -> f64 {
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
    	print!("> ");
    	io::stdout().flush().ok().expect("Could not flush stdout");
    	let mut line = String::new();
    	stdin.read_line(&mut line).ok().expect("Unable to read from stdin");
    	let result = interp.exec(String::from_str(line.trim_right()));
    	println!("{}", result);
    }
}
