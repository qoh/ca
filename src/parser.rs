use super::tokenizer::{Token, Symbol, Tokenizer};

use std::fmt;
use std::iter::{Iterator, Peekable};

#[derive(Debug, PartialEq)]
pub enum Expr {
	Integer(i32),
	BinaryExpr(Box<Expr>, Op, Box<Expr>)

}

#[derive(Debug, PartialEq)]
pub enum Op {
	Add,
	Subtract,
	Multiply,
	Divide
}

impl fmt::Display for Expr {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			&Expr::Integer(ref i) => write!(f, "{}", i),
			_ => write!(f, "{:?}", self)
		}
	}
}

fn get_precedence(symbol: &Symbol) -> u8 {
	match symbol {
		&Symbol::Add | &Symbol::Subtract => 10,
		&Symbol::Multiply | &Symbol::Divide => 20,
	}
}

pub fn parse(tokens: Vec<Token>) -> Result<Expr, String> {
	let mut it = tokens.iter().peekable();
	parse_expr(&mut it, 0)
}

fn parse_expr<'a, It>(it: &mut Peekable<It>, precedence: u8) -> Result<Expr, String>
	where It: Iterator<Item=&'a Token> {

	let mut expr = parse_prefix(it).unwrap();

	while let Some(&next_token) = it.peek() {
		let next_precedence = match next_token {
			&Token::Operator(ref symbol) => get_precedence(symbol),
			_ => { return Err(String::from("Expected operator after expression")) }
		};

		if precedence >= next_precedence {
			break;
		}

		expr = parse_infix(expr, it, next_precedence).unwrap();
	}

	Ok(expr)
}

fn parse_prefix<'a, It>(it: &mut Peekable<It>) -> Result<Expr, String>
	where It: Iterator<Item=&'a Token> {

	match it.next() {
		Some(t) => match t {
			&Token::Integer(n) => {
				Ok(Expr::Integer(n))
			},
			_ => Err(format!("Unexpected token: {:?}", t))
		},
		None => Err(String::from("No more tokens"))
	}
}

fn parse_infix<'a, It>(left: Expr, it: &mut Peekable<It>, precedence: u8) -> Result<Expr, String>
	where It: Iterator<Item=&'a Token> {

	match it.next() {
		Some(t) => match t {
			&Token::Operator(ref s) => {
				let op = match s {
					&Symbol::Add => Op::Add,
					&Symbol::Subtract => Op::Subtract,
					&Symbol::Multiply => Op::Multiply,
					&Symbol::Divide => Op::Divide,
				};

				let right = parse_expr(it, precedence).unwrap();

				Ok(Expr::BinaryExpr(
					Box::new(left),
					op,
					Box::new(right)))
			},
			_ => Err(format!("Unexpected token: {:?}", t))
		},
		None => Err(String::from("No more tokens"))
	}
}
