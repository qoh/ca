// #![feature(box_patterns)]

extern crate num;
extern crate rustyline;

use rustyline::error::ReadlineError;
use rustyline::Editor;

use std::env;

mod parser;
mod tokenizer;
mod evaluator;
mod context;

fn main() {
	let mut scope = context::Scope::new();

	if let Some(line) = env::args().nth(1) {
		input(&String::from(line), &mut scope, false);
		return;
	}
    
	let mut rl = Editor::<()>::new();

	loop {
		let readline = rl.readline("% ");
		match readline {
			Ok(line) => {
				rl.add_history_entry(&line);
				input(&line, &mut scope, true);
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

fn input(line: &String, scope: &mut context::Scope, align: bool) {
    let tokens = match tokenizer::tokenize(&line) {
        Ok(e) => e,
        Err(e) => { println!("Error: {}", e); return }
    };
    let expression = match parser::parse(tokens) {
        Ok(e) => e,
        Err(e) => { println!("Error: {}", e); return }
    };

    let mut context = context::Context::new(scope);

    if let parser::Expr::Assign(lhs, rhs) = expression {
        if let parser::Expr::Name(ref name) = *lhs {
            context.insert((*name).clone(), *rhs);
        } else {
            println!("Error: Cannot assign to {}", lhs);
        }
        return;
    }

    let expression = evaluator::evaluate(expression, &mut context).ok().unwrap();

    if align {
        print!("  ");
    }

    print!("{}", expression);

    // Print fraction of numbers
    if let parser::Expr::Number(ref n) = expression {
        if !n.is_integer() {
            print!("  ({:#})", expression);
        }
    }

    println!("");
}
