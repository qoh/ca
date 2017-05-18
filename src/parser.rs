use super::tokenizer::{Token, Symbol};

use std::fmt;
use std::iter::{Iterator, Peekable};

use num::{Zero, Signed, Integer, BigRational};
use num::bigint::{Sign, ToBigInt};

#[derive(Debug, PartialEq)]
pub enum Expr {
	Number(BigRational),
	Name(String),
	Boolean(bool),
	BinaryExpr(Box<Expr>, Op, Box<Expr>)
}

#[derive(Debug, PartialEq)]
pub enum Op {
	Add,
	Subtract,
	Multiply,
	Adjacent,
	Divide,
	Modulus,
	Exponent,
	Equals
}

impl fmt::Display for Expr {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			&Expr::Number(ref i) => {
				if f.alternate() {
					i.fmt(f)
				} else {
					fmt_ratio_decimal(i, f)
				}
			},
			&Expr::Name(ref n) => write!(f, "{}", n),
			&Expr::Boolean(ref b) => write!(f, "{}", b),
			&Expr::BinaryExpr(ref lhs, Op::Adjacent, ref rhs) => {
				write!(f, "(")?;
				lhs.fmt(f)?;
				write!(f, " ")?;
				rhs.fmt(f)?;
				write!(f, ")")
			},
			&Expr::BinaryExpr(ref lhs, ref op, ref rhs) => {
				write!(f, "(")?;
				lhs.fmt(f)?;
				write!(f, " ")?;
				op.fmt(f)?;
				write!(f, " ")?;
				rhs.fmt(f)?;
				write!(f, ")")
			}
		}
	}
}

fn fmt_ratio_decimal(r: &BigRational, f: &mut fmt::Formatter) -> fmt::Result {
	let precision = f.precision().unwrap_or(5);
	let base = 10.to_bigint().unwrap();

	let num = r.numer();
	let den = r.denom();

	if num.sign() == Sign::Minus { write!(f, "-")?; }

	let mut div = num.abs().div_rem(den);
	write!(f, "{}.", div.0)?;

	for _ in 0..precision {
		if div.1.is_zero() { break }
		div = (&base * div.1).div_rem(den);
		write!(f, "{}", div.0)?;
	}

	if !div.1.is_zero() {
		write!(f, "...")
	} else {
		Ok(())
	}
}

impl fmt::Display for Op {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			&Op::Add => write!(f, "+"),
			&Op::Subtract => write!(f, "-"),
			&Op::Multiply => write!(f, "*"),
			&Op::Adjacent => write!(f, "*"),
			&Op::Divide => write!(f, "/"),
			&Op::Modulus => write!(f, "%"),
			&Op::Exponent => write!(f, "^"),
			&Op::Equals => write!(f, "="),
		}
	}
}

const UNARY_PRIORITY: u8 = 8;

fn get_precedence(symbol: &Symbol) -> (u8, u8) {
	match symbol {
		&Symbol::Equals => (3, 3),
		&Symbol::Add | &Symbol::Subtract => (6, 6),
		&Symbol::Multiply | &Symbol::Divide | &Symbol::Modulus => (7, 7),
		&Symbol::Exponent => (10, 9),
	}
}

pub fn parse(tokens: Vec<Token>) -> Result<Expr, String> {
	let mut it = tokens.iter().peekable();
	parse_expr(&mut it, 0)
}

fn parse_expr<'a, It>(it: &mut Peekable<It>, precedence: u8) -> Result<Expr, String>
	where It: Iterator<Item=&'a Token> {

	let mut expr = parse_prefix(it)?;

	while let Some(&next_token) = it.peek() {
		let (left_prec, right_prec) = match next_token {
			&Token::Operator(ref symbol) => get_precedence(symbol),
			&Token::LeftParen => break,
			&Token::RightParen => break,
			_ => {
				expr = Expr::BinaryExpr(
					Box::new(expr),
					Op::Adjacent,
					Box::new(parse_expr(it, 0)?) // FIXME: Is 0 the right precedence for this?
				);
				continue; // FIXME: Continue? Shouldn't this consume everything possible?
			}
		};

		if precedence >= left_prec {
			break;
		}

		expr = parse_infix(expr, it, right_prec)?;
	}

	Ok(expr)
}

fn parse_prefix<'a, It>(it: &mut Peekable<It>) -> Result<Expr, String>
	where It: Iterator<Item=&'a Token> {

	// TODO: Don't use Clone here
	match it.next() {
		Some(t) => match t {
			&Token::Integer(ref n) => {
				Ok(Expr::Number(n.clone()))
			},
			&Token::Name(ref n) => {
				Ok(Expr::Name(n.clone()))
			},
			&Token::Operator(Symbol::Subtract) => {
				Ok(Expr::BinaryExpr(
					Box::new(Expr::Number(BigRational::zero())),
					Op::Subtract,
					Box::new(parse_expr(it, UNARY_PRIORITY)?)))
			},
			&Token::LeftParen => {
				let result = parse_expr(it, 0);
				match it.next() {
					Some(&Token::RightParen) => result,
					_ => Err(String::from("Missing right parenthesis"))
				}
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

				let right = parse_expr(it, precedence)?;

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
