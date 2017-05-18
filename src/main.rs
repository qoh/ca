// #![feature(box_patterns)]

extern crate num;
extern crate rustyline;

use rustyline::error::ReadlineError;
use rustyline::Editor;

mod parser;
mod tokenizer;
mod evaluator;

use tokenizer::Tokenizer;

fn main() {
	let mut rl = Editor::<()>::new();

	loop {
		let readline = rl.readline("% ");
		match readline {
			Ok(line) => {
				rl.add_history_entry(&line);
				let tokens = line.tokenize();
				let expression = parser::parse(tokens).ok().unwrap();
				let expression = evaluator::evaluate(expression).ok().unwrap();
				println!("  {}", expression);
			},
			Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => {
				break
			},
			Err(err) => {
				println!("Error: {:?}", err);
				break
			}
		}
	}
}
