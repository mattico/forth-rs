use std::rc::Rc;

use dictionary::Entry;

pub type Int = i32;
pub type Double = f64;

#[derive(Clone, Debug)]
pub enum ForthCell {
	Number(Int),
	Word(Rc<Entry>),
}