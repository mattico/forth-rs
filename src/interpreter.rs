
use types::{Int, ForthCell};
use error::{ForthResult, ForthError};
use dictionary::{Dictionary, DictionaryExt, Entry};
use statement::{Statement, StatementExt};
use word::ForthWord;

pub type Stack = Vec<Int>;

pub struct Interpreter {
    pub dictionary: Dictionary,
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
				ForthWord::Native(Box::new(|_: &mut Interpreter, stack: &mut Stack| -> ForthResult<()> {
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
				ForthWord::Native(Box::new(|_: &mut Interpreter, stack: &mut Stack| -> ForthResult<()> {
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
				ForthWord::Native(Box::new(|_: &mut Interpreter, _: &mut Stack| -> ForthResult<()> {
					ForthCell::Number($o);
					Ok(())
				})),
			)
	}
}

impl Interpreter {
	pub fn new() -> Interpreter {
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
				ForthWord::Native(Box::new(|interp: &mut Interpreter, stack: &mut Stack| -> ForthResult<()> {
					let x = try_stack!(interp.last_result.clone());
					for v in x {
						stack.push(v);
					}
					Ok(())
				})),
			));

		dict.insert_entry(Entry::new("<",
				ForthWord::Native(Box::new(|_: &mut Interpreter, stack: &mut Stack| -> ForthResult<()> {
					let y = try_stack!(stack.pop());
					let x = try_stack!(stack.pop());
					stack.push(if x < y { 1 } else { 0 });
					Ok(())
				})),
			));

		dict.insert_entry(Entry::new(">",
				ForthWord::Native(Box::new(|_: &mut Interpreter, stack: &mut Stack| -> ForthResult<()> {
					let y = try_stack!(stack.pop());
					let x = try_stack!(stack.pop());
					stack.push(if x > y { 1 } else { 0 });
					Ok(())
				})),
			));

		dict.insert_entry(Entry::new("=",
				ForthWord::Native(Box::new(|_: &mut Interpreter, stack: &mut Stack| -> ForthResult<()> {
					let y = try_stack!(stack.pop());
					let x = try_stack!(stack.pop());
					stack.push(if x == y { 1 } else { 0 });
					Ok(())
				})),
			));

		dict.insert_entry(Entry::new("0<",
				ForthWord::Native(Box::new(|_: &mut Interpreter, stack: &mut Stack| -> ForthResult<()> {
					let x = try_stack!(stack.pop());
					stack.push(if x < 0 { 1 } else { 0 });
					Ok(())
				})),
			));

		dict.insert_entry(Entry::new("0=",
				ForthWord::Native(Box::new(|_: &mut Interpreter, stack: &mut Stack| -> ForthResult<()> {
					let x = try_stack!(stack.pop());
					stack.push(if x == 0 { 1 } else { 0 });
					Ok(())
				})),
			));

		dict.insert_entry(Entry::new("0>",
				ForthWord::Native(Box::new(|_: &mut Interpreter, stack: &mut Stack| -> ForthResult<()> {
					let x = try_stack!(stack.pop());
					stack.push(if x > 0 { 1 } else { 0 });
					Ok(())
				})),
			));

		dict.insert_entry(Entry::new("1+",
				ForthWord::Native(Box::new(|_: &mut Interpreter, stack: &mut Stack| -> ForthResult<()> {
					let x = try_stack!(stack.pop());
					stack.push(x + 1);
					Ok(())
				})),
			));

		dict.insert_entry(Entry::new("1-",
				ForthWord::Native(Box::new(|_: &mut Interpreter, stack: &mut Stack| -> ForthResult<()> {
					let x = try_stack!(stack.pop());
					stack.push(x - 1);
					Ok(())
				})),
			));

		dict.insert_entry(Entry::new("2+",
				ForthWord::Native(Box::new(|_: &mut Interpreter, stack: &mut Stack| -> ForthResult<()> {
					let x = try_stack!(stack.pop());
					stack.push(x + 2);
					Ok(())
				})),
			));

		dict.insert_entry(Entry::new("2-",
				ForthWord::Native(Box::new(|_: &mut Interpreter, stack: &mut Stack| -> ForthResult<()> {
					let x = try_stack!(stack.pop());
					stack.push(x - 2);
					Ok(())
				})),
			));

		dict.insert_entry(Entry::new("2/",
				ForthWord::Native(Box::new(|_: &mut Interpreter, stack: &mut Stack| -> ForthResult<()> {
					let x = try_stack!(stack.pop());
					stack.push(x >> 1); // Per fst83 standard
					Ok(())
				})),
			));

		dict.insert_entry(Entry::new("dup",
				ForthWord::Native(Box::new(|_: &mut Interpreter, stack: &mut Stack| -> ForthResult<()> {
					let x = try_stack!(stack.pop());
					stack.push(x.clone());
					stack.push(x);
					Ok(())
				})),
			));

		dict.insert_entry(Entry::new("?dup",
				ForthWord::Native(Box::new(|_: &mut Interpreter, stack: &mut Stack| -> ForthResult<()> {
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
				ForthWord::Native(Box::new(|_: &mut Interpreter, stack: &mut Stack| -> ForthResult<()> {
					let x = try_stack!(stack.pop());
					let y = try_stack!(stack.pop());
					stack.push(x.clone());
					stack.push(y);
					stack.push(x);
					Ok(())
				})),
			));

		dict.insert_entry(Entry::new("swap",
				ForthWord::Native(Box::new(|_: &mut Interpreter, stack: &mut Stack| -> ForthResult<()> {
					let x = try_stack!(stack.pop());
					let y = try_stack!(stack.pop());
					stack.push(x);
					stack.push(y);
					Ok(())
				})),
			));

		dict.insert_entry(Entry::new("rot",
				ForthWord::Native(Box::new(|_: &mut Interpreter, stack: &mut Stack| -> ForthResult<()> {
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
				ForthWord::Native(Box::new(|_: &mut Interpreter, stack: &mut Stack| -> ForthResult<()> {
					println!("ds =  {:?} ", stack);
					Ok(())
				})),
			));

		dict.insert_entry(Entry::new("cr",
				ForthWord::Native(Box::new(|_: &mut Interpreter, _: &mut Stack| -> ForthResult<()> {
					println!("");
					Ok(())
				})),
			));

		dict.insert_entry(Entry::new(".",
				ForthWord::Native(Box::new(|_: &mut Interpreter, stack: &mut Stack| -> ForthResult<()> {
					println!("{}", try_stack!(stack.pop()));
					Ok(())
				})),
			));

		let square = {
			let mut s = Statement::new();
			s.push_back(ForthCell::Word(dict.get("dup").unwrap().clone()));
			s.push_back(ForthCell::Word(dict.get("*").unwrap().clone()));
			s
		};
		dict.insert_entry(Entry::from_statement("square", square));

		dict.insert_entry(Entry::new("branch",
				ForthWord::Native(Box::new(|_: &mut Interpreter, stack: &mut Stack| -> ForthResult<()> {
					let y = try_stack!(stack.pop());
					let x = try_stack!(stack.pop());
					stack.push(if x == y { 1 } else { 0 });
					Ok(())
				})),
			));

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