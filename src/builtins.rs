use dictionary::{Dictionary, DictionaryExt, Entry};
use interpreter::{Interpreter, Stack};
use statement::{Statement};
use types::ForthCell;
use error::{ForthResult, ForthError};
use word::ForthWord;

macro_rules! try_stack {
    ($x:expr) => {
        match $x {
            Some(i) => i,
            None => return Err(ForthError::EmptyStack),
        }
    }
}

macro_rules! binary_entry {
    ($dict:ident, $name:expr, $f:expr) => {
        $dict.insert_entry(Entry::new($name,
                ForthWord::Native(Box::new(|_: &mut Interpreter, stack: &mut Stack| -> ForthResult<()> {
                    let b = try_stack!(stack.pop());
                    let a = try_stack!(stack.pop());
                    stack.push($f(a, b));
                    Ok(())
                })),
            ))
    }
}

macro_rules! unary_entry {
    ($dict:ident, $name:expr, $f:expr) => {
        $dict.insert_entry(Entry::new($name,
                ForthWord::Native(Box::new(|_: &mut Interpreter, stack: &mut Stack| -> ForthResult<()> {
                    let a = try_stack!(stack.pop());
                    stack.push($f(a));
                    Ok(())
                })),
            ))
    }
}

macro_rules! nonary_entry {
    ($dict:ident, $name:expr, $f:expr) => {
        $dict.insert_entry(Entry::new($name,
                ForthWord::Native(Box::new(|_: &mut Interpreter, _: &mut Stack| -> ForthResult<()> {
                    ForthCell::Number($f);
                    Ok(())
                })),
            ))
    }
}

macro_rules! entry {
    ($dict:ident, $name:expr, $f:expr) => {
        $dict.insert_entry(Entry::new($name, ForthWord::Native(Box::new($f))))
    }
}

pub fn insert_builtins(dict: &mut Dictionary) {
    binary_entry!(dict, "/", ::std::ops::Div::div);
    binary_entry!(dict, "*", ::std::ops::Mul::mul);
    binary_entry!(dict, "+", ::std::ops::Add::add);
    binary_entry!(dict, "-", ::std::ops::Sub::sub);
    binary_entry!(dict, "mod", ::std::ops::Rem::rem);
    binary_entry!(dict, "and", ::std::ops::BitAnd::bitand);
    binary_entry!(dict, "or", ::std::ops::BitOr::bitor);
    binary_entry!(dict, "xor", ::std::ops::BitXor::bitxor);
    binary_entry!(dict, "rshift", ::std::ops::Shr::shr);
    binary_entry!(dict, "lshift", ::std::ops::Shl::shl);
    binary_entry!(dict, "max", ::std::cmp::max);
    binary_entry!(dict, "min", ::std::cmp::min);

    unary_entry!(dict, "negate", ::std::ops::Neg::neg);
    unary_entry!(dict, "not", ::std::ops::Not::not);

    nonary_entry!(dict, "bye", ::std::process::exit(0));

    entry!(dict, "*/", |_: &mut Interpreter, stack: &mut Stack| -> ForthResult<()> {
                let z = try_stack!(stack.pop()) as i64; // double-length intermediate result
                let y = try_stack!(stack.pop()) as i64;
                let x = try_stack!(stack.pop()) as i64;
                stack.push((x * y / z) as i32);
                Ok(())
            });

    entry!(dict, "*/mod", |_: &mut Interpreter, stack: &mut Stack| -> ForthResult<()> {
                let z = try_stack!(stack.pop()) as i64; // double-length intermediate result
                let y = try_stack!(stack.pop()) as i64;
                let x = try_stack!(stack.pop()) as i64;
                stack.push((x * y % z) as i32);
                stack.push((x * y / z) as i32);
                Ok(())
            });


    entry!(dict, "abs", |_: &mut Interpreter, stack: &mut Stack| -> ForthResult<()> {
                let x = try_stack!(stack.pop());
                stack.push(x.abs());
                Ok(())
            });

    entry!(dict, "$", |interp: &mut Interpreter, stack: &mut Stack| -> ForthResult<()> {
                let x = try_stack!(interp.last_result.clone());
                for v in x {
                    stack.push(v);
                }
                Ok(())
            });

    entry!(dict, "<", | _: &mut Interpreter, stack: &mut Stack| -> ForthResult<()> {
                let y = try_stack!(stack.pop());
                let x = try_stack!(stack.pop());
                stack.push(if x < y { 1 } else { 0 });
                Ok(())
            });

    entry!(dict, ">", |_: &mut Interpreter, stack: &mut Stack| -> ForthResult<()> {
                let y = try_stack!(stack.pop());
                let x = try_stack!(stack.pop());
                stack.push(if x > y { 1 } else { 0 });
                Ok(())
            });

    entry!(dict, "=", |_: &mut Interpreter, stack: &mut Stack| -> ForthResult<()> {
                let y = try_stack!(stack.pop());
                let x = try_stack!(stack.pop());
                stack.push(if x == y { 1 } else { 0 });
                Ok(())
            });

    entry!(dict, "0<", |_: &mut Interpreter, stack: &mut Stack| -> ForthResult<()> {
                let x = try_stack!(stack.pop());
                stack.push(if x < 0 { 1 } else { 0 });
                Ok(())
            });

    entry!(dict, "0=", |_: &mut Interpreter, stack: &mut Stack| -> ForthResult<()> {
                let x = try_stack!(stack.pop());
                stack.push(if x == 0 { 1 } else { 0 });
                Ok(())
            });

    entry!(dict, "0>", |_: &mut Interpreter, stack: &mut Stack| -> ForthResult<()> {
                let x = try_stack!(stack.pop());
                stack.push(if x > 0 { 1 } else { 0 });
                Ok(())
            });

    entry!(dict, "1+", |_: &mut Interpreter, stack: &mut Stack| -> ForthResult<()> {
                let x = try_stack!(stack.pop());
                stack.push(x + 1);
                Ok(())
            });

    entry!(dict, "1-", |_: &mut Interpreter, stack: &mut Stack| -> ForthResult<()> {
                let x = try_stack!(stack.pop());
                stack.push(x - 1);
                Ok(())
            });

    entry!(dict, "2+", |_: &mut Interpreter, stack: &mut Stack| -> ForthResult<()> {
                let x = try_stack!(stack.pop());
                stack.push(x + 2);
                Ok(())
            });

    entry!(dict, "2-", |_: &mut Interpreter, stack: &mut Stack| -> ForthResult<()> {
                let x = try_stack!(stack.pop());
                stack.push(x - 2);
                Ok(())
            });

    entry!(dict, "2/", |_: &mut Interpreter, stack: &mut Stack| -> ForthResult<()> {
                let x = try_stack!(stack.pop());
                stack.push(x >> 1); // Per fst83 standard
                Ok(())
            });

    entry!(dict, "2*", |_: &mut Interpreter, stack: &mut Stack| -> ForthResult<()> {
                let x = try_stack!(stack.pop());
                stack.push(x << 1); // Per fst83 standard
                Ok(())
            });

    entry!(dict, "dup", |_: &mut Interpreter, stack: &mut Stack| -> ForthResult<()> {
                let x = try_stack!(stack.pop());
                stack.push(x.clone());
                stack.push(x);
                Ok(())
            });

    entry!(dict, "?dup", |_: &mut Interpreter, stack: &mut Stack| -> ForthResult<()> {
                let x = try_stack!(stack.pop());
                if x != 0 {
                    stack.push(x.clone());
                    stack.push(x);
                } else {
                    stack.push(x);
                }
                Ok(())
            });

    entry!(dict, "drop", |_: &mut Interpreter, stack: &mut Stack| -> ForthResult<()> {
                let _ = try_stack!(stack.pop());
                Ok(())
            });

    entry!(dict, "over", |_: &mut Interpreter, stack: &mut Stack| -> ForthResult<()> {
                let x = try_stack!(stack.pop());
                let y = try_stack!(stack.pop());
                stack.push(x.clone());
                stack.push(y);
                stack.push(x);
                Ok(())
            });

    entry!(dict, "swap", |_: &mut Interpreter, stack: &mut Stack| -> ForthResult<()> {
                let x = try_stack!(stack.pop());
                let y = try_stack!(stack.pop());
                stack.push(x);
                stack.push(y);
                Ok(())
            });

    entry!(dict, "rot", |_: &mut Interpreter, stack: &mut Stack| -> ForthResult<()> {
                let x = try_stack!(stack.pop());
                let y = try_stack!(stack.pop());
                let z = try_stack!(stack.pop());
                stack.push(y);
                stack.push(x);
                stack.push(z);
                Ok(())
            });

    entry!(dict, "dump", |_: &mut Interpreter, stack: &mut Stack| -> ForthResult<()> {
                println!("ds =  {:?} ", stack);
                Ok(())
            });

    entry!(dict, "cr", |_: &mut Interpreter, _: &mut Stack| -> ForthResult<()> {
                println!("");
                Ok(())
            });

    entry!(dict, ".", |_: &mut Interpreter, stack: &mut Stack| -> ForthResult<()> {
                println!("{}", try_stack!(stack.pop()));
                Ok(())
            });

    entry!(dict, "branch", |_: &mut Interpreter, stack: &mut Stack| -> ForthResult<()> {
                let y = try_stack!(stack.pop());
                let x = try_stack!(stack.pop());
                stack.push(if x == y { 1 } else { 0 });
                Ok(())
            });

    entry!(dict, "emit", |_: &mut Interpreter, stack: &mut Stack| -> ForthResult<()> {
                let x = try_stack!(stack.pop());
                if x < 0 || x > 256 { return Err(ForthError::InvalidCharacter) }
                print!("{}", x as u8 as char);
                Ok(())
            });

    let square = {
        let mut s = Statement::new();
        s.push_back(ForthCell::Word(dict.get("dup").unwrap().clone()));
        s.push_back(ForthCell::Word(dict.get("*").unwrap().clone()));
        s
    };
    dict.insert_entry(Entry::from_statement("square", square));
}