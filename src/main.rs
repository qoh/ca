// #![feature(box_patterns)]

extern crate num;
extern crate rustyline;

use rustyline::error::ReadlineError;
use rustyline::Editor;

mod parser;
mod tokenizer;
mod evaluator;

fn main() {
	let mut rl = Editor::<()>::new();

	loop {
		let readline = rl.readline("% ");
		match readline {
			Ok(line) => {
				rl.add_history_entry(&line);

				let tokens = match tokenizer::tokenize(&line) {
					Ok(e) => e,
					Err(e) => { println!("Error: {}", e); continue }
				};
				let expression = match parser::parse(tokens) {
					Ok(e) => e,
					Err(e) => { println!("Error: {}", e); continue }
				};

				// println!(" >{:#}", expression);
				let expression = evaluator::evaluate(expression).ok().unwrap();

				print!("  {}", expression);
				if let parser::Expr::Number(_) = expression {
					println!("  ({:#})", expression);
				} else {
					println!("");
				}
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
