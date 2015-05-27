use std::rc::Rc;

use dictionary::Entry;

pub type Int = i32;
pub type Double = f64;

pub const TRUE: i32 = 0xFFFFFFFF_u32 as i32;
pub const FALSE: i32 = 0x00000000_u32 as i32;

#[derive(Clone, Debug)]
pub enum ForthCell {
    Number(Int),
    Word(Rc<Entry>),
}