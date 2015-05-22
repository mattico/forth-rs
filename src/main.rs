use std::io;
use std::io::prelude::*;

extern crate forth;

use forth::interpreter::Interpreter;

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
