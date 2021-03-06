pub mod types;
pub mod symbols;
pub mod interpreter;
pub mod parser;
pub mod instruction;
pub mod user;
pub mod server;
pub mod mail;
pub mod error;
pub mod environment;
pub mod modifier;
extern crate regex;

use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::env;

fn run(fname: &str) {
	let path = Path::new(&fname);
	let display = path.display();

	let mut file = match File::open(path) {
		Ok(val) => val,
		Err(err) => panic!("couldn't open {}: {}", display, err.description())
	};

	let mut contents = String::new();
	match file.read_to_string(&mut contents) {
		Err(why) => panic!("couldn't read {}: {}", display, why.description()),
		Ok(_) => {}
	};

	let instructions = match parser::parse(&contents) {
		Ok(val) => val,
		Err(err) => {
			println!("{}", err);
			if let Some(ref pos) = err.pos {
				if let Some(ref s) = contents.lines().nth(pos.0 - 1) {
					// let new_s = s.replace('\t', "    ");
					let ltrim = s.trim_left();
					let lspace = &s[0..ltrim.len()];
					let actual_lspace = lspace.replace('\t', "    ");
					let dstr = &s[0..pos.1];
					let dashed_lspace = dstr.chars()
					                        .map(|c|if c == '\t' {"----"} else {"-"})
					                        .collect::<String>();
					println!("{}{}", actual_lspace, ltrim);
					println!("{}^", dashed_lspace);
				}
			}
			return;
		}
	};
	let mut inter = interpreter::Interpreter::new();

	inter.execute(&instructions);

	println!("");
}

fn help() {
	println!(
r"      _ _ _ _  __________       _ _
    /_/_/_/_/ | @≈≈   /  |     (_| |
   /_/_ _  _  |  /\  /\  | __ _ _| |     __ _ _ __   __ _
  /_/_/_/ |_| | //\\//\\ |/ _` | | |    / _` | '_ \ / _` |
 /_/_ _ _     |//  \/  \\| (_| | | |___| (_|_| | | | (_| |
/_/_/_/_/     |/________\|\__,_|_\_____/\/   |_|_|_|\__, |
A programming language based on emails   \    ___________|
                                          \__|      v1.0.0
See DOC.md for documentation on how to use Emailang.
Alternatively, see README.md for a quick tutorial.
emailang <file> - run the given file");
}

fn main() {
	let args = env::args().collect::<Vec<String>>();
	match args.len() {
		0 => panic!("This should not be possible!"),
		1 => help(),
		2 => run(&args[1]),
		_ => println!("Invalid number of arguments!")
	}
}
