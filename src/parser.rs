use super::tokenizer::{Token, Symbol};

use std::fmt;
use std::iter::{Iterator, Peekable};

use num::BigRational;

#[derive(Debug, PartialEq)]
pub enum Expr {
	Number(BigRational),
	BinaryExpr(Box<Expr>, Op, Box<Expr>)

}

#[derive(Debug, PartialEq)]
pub enum Op {
	Add,
	Subtract,
	Multiply,
	Divide,
	Modulus,
	Exponent,
	Equals
}

impl fmt::Display for Expr {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			&Expr::Number(ref i) => write!(f, "{}", i),
			&Expr::BinaryExpr(ref lhs, ref op, ref rhs) => write!(f, "({} {} {})", lhs, op, rhs)
		}
	}
}

impl fmt::Display for Op {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			&Op::Add => write!(f, "+"),
			&Op::Subtract => write!(f, "-"),
			&Op::Multiply => write!(f, "*"),
			&Op::Divide => write!(f, "/"),
			&Op::Modulus => write!(f, "%"),
			&Op::Exponent => write!(f, "^"),
			&Op::Equals => write!(f, "="),
		}
	}
}

fn get_precedence(symbol: &Symbol) -> u8 {
	match symbol {
		&Symbol::Equals => 5,
		&Symbol::Add | &Symbol::Subtract => 10,
		&Symbol::Multiply | &Symbol::Divide | &Symbol::Modulus => 20,
		&Symbol::Exponent => 30, // TODO: needs to be right-associative
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
			&Token::Integer(ref n) => {
				Ok(Expr::Number(n.clone()))
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
					&Symbol::Modulus => Op::Modulus,
					&Symbol::Exponent => Op::Exponent,
					&Symbol::Equals => Op::Equals,
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
