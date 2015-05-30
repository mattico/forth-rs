use dictionary::{Dictionary, DictionaryExt, Entry};
use types::{self, ForthCell, Int, Statement};
use error::{ForthResult, ForthError};
use word::ForthWord;

macro_rules! try_pop {
    ($interp:ident) => (try!($interp.parameter_stack.pop().ok_or(ForthError::EmptyStack)))
}

macro_rules! entry {
    ($dict:ident, $name:expr, $f:expr) => (
        $dict.insert_entry(Entry::new($name, ForthWord::Native(Box::new($f))));
    )
}

macro_rules! unary_entry {
    ($dict:ident, $name:expr, $f:expr) => ( entry!($dict, $name, |interp| -> ForthResult<()> {
        let a = try_pop!(interp);
        interp.parameter_stack.push($f(a));
        Ok(())
    }))
}

macro_rules! binary_entry {
    ($dict:ident, $name:expr, $f:expr) => ( entry!($dict, $name, |interp| -> ForthResult<()> {
        let b = try_pop!(interp);
        let a = try_pop!(interp);
        interp.parameter_stack.push($f(a, b));
        Ok(())
    }))
}

macro_rules! trinary_entry {
    ($dict:ident, $name:expr, $f:expr) => ( entry!($dict, $name, |interp| -> ForthResult<()> {
        let c = try_pop!(interp);
        let b = try_pop!(interp);
        let a = try_pop!(interp);
        interp.parameter_stack.push($f(a, b, c));
        Ok(())
    }))
}

pub fn insert_builtins(dict: &mut Dictionary) {
    entry!(dict, "bye", |_| { ::std::process::exit(0) });

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
    unary_entry!(dict, "abs", |x: Int| { x.abs() });

    binary_entry!(dict, "<", |x, y| { if x < y { types::TRUE } else { types::FALSE } });
    binary_entry!(dict, ">", |x, y| { if x > y { types::TRUE } else { types::FALSE } });
    binary_entry!(dict, "=", |x, y| { if x == y { types::TRUE } else { types::FALSE } });

    unary_entry!(dict, "0<", |x| { if x < 0 { types::TRUE } else { types::FALSE } });
    unary_entry!(dict, "0>", |x| { if x > 0 { types::TRUE } else { types::FALSE } });
    unary_entry!(dict, "0=", |x| { if x == 0 { types::TRUE } else { types::FALSE } });

    unary_entry!(dict, "1+", |x| { x + 1 });
    unary_entry!(dict, "1-", |x| { x - 1 });
    unary_entry!(dict, "2+", |x| { x + 2 });
    unary_entry!(dict, "2-", |x| { x - 2 });
    unary_entry!(dict, "2/", |x| { x / 2 });
    unary_entry!(dict, "2*", |x| { x * 2 });

    trinary_entry!(dict, "*/", |x, y, z| {
        (x as i64 * y as i64 / z as i64) as Int
    });

    entry!(dict, "*/mod", |interp| -> ForthResult<()> {
        let z = try_pop!(interp);
        let y = try_pop!(interp);
        let x = try_pop!(interp);
        interp.parameter_stack.push((x as i64 * y as i64 % z as i64) as Int);
        interp.parameter_stack.push((x as i64 * y as i64 / z as i64) as Int);
        Ok(())
    });

    entry!(dict, "drop", |interp| {
        let _ = try_pop!(interp);
        Ok(())
    });

    entry!(dict, "dup", |interp| -> ForthResult<()> {
        let x = try_pop!(interp);
        interp.parameter_stack.push(x.clone());
        interp.parameter_stack.push(x);
        Ok(())
    });

    entry!(dict, "?dup", |interp| -> ForthResult<()> {
        let x = try_pop!(interp);
        if x != 0 {
            interp.parameter_stack.push(x.clone());
            interp.parameter_stack.push(x);
        } else {
            interp.parameter_stack.push(0);
        }
        Ok(())
    });

    entry!(dict, "over", |interp| -> ForthResult<()> {
        let x = try_pop!(interp);
        let y = try_pop!(interp);
        interp.parameter_stack.push(x.clone());
        interp.parameter_stack.push(y);
        interp.parameter_stack.push(x);
        Ok(())
    });

    entry!(dict, "swap", |interp| -> ForthResult<()> {
        let x = try_pop!(interp);
        let y = try_pop!(interp);
        interp.parameter_stack.push(x);
        interp.parameter_stack.push(y);
        Ok(())
    });

    entry!(dict, "rot", |interp| -> ForthResult<()> {
        let x = try_pop!(interp);
        let y = try_pop!(interp);
        let z = try_pop!(interp);
        interp.parameter_stack.push(y);
        interp.parameter_stack.push(x);
        interp.parameter_stack.push(z);
        Ok(())
    });

    entry!(dict, "branch", |interp| -> ForthResult<()> {
        interp.statement_index += 1;
        let string = try!(interp.get_current().ok_or(ForthError::ExpectedNumber));
        if let Ok(n) = string.parse::<Int>() {
            try!(interp.jump(n));
        } else {
            return Err(ForthError::ExpectedNumber);
        }
        Ok(())
    });

    entry!(dict, "?branch", |interp| -> ForthResult<()> {
        let x = try_pop!(interp);
        if x == types::TRUE {
            try!(interp.next());
            let string = try!(interp.get_current().ok_or(ForthError::ExpectedNumber));
            if let Ok(n) = string.parse::<Int>() {
                try!(interp.jump(n));
            } else {
                return Err(ForthError::ExpectedNumber);
            }
        }
        Ok(())
    });

    entry!(dict, "emit", |interp| -> ForthResult<()> {
        let x = try_pop!(interp);
        if x < 0 || x > 256 { return Err(ForthError::InvalidCharacter) }
        print!("{}", x as u8 as char);
        Ok(())
    });

    entry!(dict, "dump", |interp| -> ForthResult<()> {
        println!("ds =  {:?} ", interp.parameter_stack);
        Ok(())
    });

    entry!(dict, "cr", |_| -> ForthResult<()> {
        println!("");
        Ok(())
    });

    entry!(dict, ".", |interp| -> ForthResult<()> {
        print!("{}", try_pop!(interp));
        Ok(())
    });

    entry!(dict, ":", |interp| -> ForthResult<()> {
        let mut stmt = Statement::new();
        try!(interp.next());
        let name = try!((*interp).get_current().ok_or(ForthError::WordNameNotFound));
        try!(interp.next());
        while let Some(w) = interp.get_current() {
            if w == ";" { break; }
            stmt.push_back(try!(interp.parse_word(w)));
            try!(interp.next());
        }
        interp.dictionary.insert_entry(Entry::from_statement(name.as_str(), stmt));
        Ok(())
    });

    entry!(dict, ";", |_| -> ForthResult<()> {
        Err(ForthError::SemicolonOutsideOfWordDefinition) // Jeez
    });

    // To make sure that forth words work
    let square = {
        let mut s = Statement::new();
        s.push_back(ForthCell::Word(dict.get("dup").unwrap().clone()));
        s.push_back(ForthCell::Word(dict.get("*").unwrap().clone()));
        s
    };
    dict.insert_entry(Entry::from_statement("square", square));
}