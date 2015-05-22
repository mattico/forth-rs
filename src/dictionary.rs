use std::collections::HashMap;
use std::rc::Rc;
use std::fmt;
use std::hash::{Hash, Hasher};

use word::ForthWord;
use statement::Statement;

pub type Dictionary = HashMap<String, Rc<Entry>>;

pub trait DictionaryExt {
	fn insert_entry(&mut self, Entry);
}

impl DictionaryExt for Dictionary {
	fn insert_entry(&mut self, entry: Entry) {
		self.insert(entry.name.clone(), Rc::new(entry));
	}
}

#[derive(Clone)]
pub struct Entry {
	pub name: String,
	pub code: Rc<ForthWord>,
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
	pub fn new(name: &str, code: ForthWord) -> Entry {
		Entry {
			name: String::from_str(name),
			code: Rc::new(code),
		}
	}

	pub fn from_statement(name: &str, stmt: Statement) -> Entry {
		Entry {
			name: name.to_string(),
			code: Rc::new(ForthWord::Forth(stmt)),
		}
	}
}