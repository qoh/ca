// #![feature(box_patterns)]

extern crate num;
extern crate rustyline;

use rustyline::error::ReadlineError;
use rustyline::Editor;

mod parser;
mod tokenizer;
mod evaluator;
mod context;

fn main() {
	let mut rl = Editor::<()>::new();

	let mut scope = context::Scope::new();

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

				let mut context = context::Context::new(&mut scope);

				if let parser::Expr::Assign(lhs, rhs) = expression {
					if let parser::Expr::Name(ref name) = *lhs {
						context.insert((*name).clone(), *rhs);
					} else {
						println!("Error: Cannot assign to {}", lhs);
					}
					continue;
				}

				let expression = evaluator::evaluate(expression, &mut context).ok().unwrap();

				print!("  {}", expression);

				// Print fraction of numbers
				if let parser::Expr::Number(ref n) = expression {
					if !n.is_integer() {
						println!("  ({:#})", expression);
						continue; // FIXME: This is not very nice. To prevent the println below.
					}
				}

				println!("");
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
