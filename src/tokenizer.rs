use std::iter::Peekable;
use std::str::Chars;

use num::{pow, BigInt, BigRational};
use num::bigint::ToBigInt;

#[derive(Debug, PartialEq)]
pub enum Token {
	Number(BigRational),
	Name(String),
	LeftParen,
	RightParen,
	Add,
	Subtract,
	Multiply,
	Divide,
	Modulus,
	Exponent,
	Equals,
	Comma,
	Assign,
}

pub fn tokenize(src: &String) -> Result<Vec<Token>, String> {
	let mut it = src.chars().peekable();
	let mut tokens: Vec<Token> = vec![];

	loop {
		match it.peek() {
			Some(&ch) => match ch {
				'0' ... '9' | '.' => {
					let num: Vec<char> = consume_while(&mut it, |a| a.is_numeric() || a == '_' || a == '.')
						.into_iter()
						.collect();
					tokens.push(Token::Number(parse_number(num)?));
				},
				'+' => {
					it.next().unwrap();
					tokens.push(Token::Add);
				},
				'-' | '−' => {
					it.next().unwrap();
					tokens.push(Token::Subtract);
				},
				'*' | '∙' => {
					it.next().unwrap();
					tokens.push(Token::Multiply);
				},
				'/' | '÷' | '∕' => {
					it.next().unwrap();
					tokens.push(Token::Divide);
				},
				'%' => {
					it.next().unwrap();
					tokens.push(Token::Modulus);
				},
				'^' => {
					it.next().unwrap();
					tokens.push(Token::Exponent);
				},
				'=' => {
					it.next().unwrap();
					tokens.push(Token::Equals);
				},
				':' => {
					it.next().unwrap();
					if let Some(&'=') = it.peek() {
						it.next().unwrap();
						tokens.push(Token::Assign);
					} else {
						return Err(String::from("Expected = after :"));
					}
				},
				'≔' => {
					it.next().unwrap();
					tokens.push(Token::Assign);
				},
				',' => { it.next().unwrap(); tokens.push(Token::Comma) },
				'(' => { it.next().unwrap(); tokens.push(Token::LeftParen) },
				')' => { it.next().unwrap(); tokens.push(Token::RightParen) },
				'\n' | '\t' | ' ' => {
					it.next().unwrap();
				},
				a if a.is_alphabetic() => {
					let name: String = consume_while(&mut it, |a| a.is_alphabetic())
						.into_iter()
						.collect();
					tokens.push(Token::Name(name));
				}
				_ => return Err(format!("Invalid char '{}'", ch))
			},
			None => break
		}
	}

	Ok(tokens)
}

fn parse_number(chars: Vec<char>) -> Result<BigRational, String> {
	// BigRational::new(chars.parse::<BigInt>().unwrap(). 1.to_bigint().unwrap())

	let mut separator: Option<usize> = None;
	let mut digits: Vec<char> = vec![];

	for a in chars.into_iter() {
		if a == '.' {
			separator = Some(digits.len());
		} else if a.is_numeric() {
			digits.push(a);
		}
	}

	let scale = match separator {
		Some(i) => digits.len() - i,
		None => 0
	};

	let denom = pow(10.to_bigint().unwrap(), scale);

	let digits: String = digits.into_iter().collect();

	if let Ok(numer) = digits.parse::<BigInt>() {
		Ok(BigRational::new(numer, denom))
	} else {
		Err(String::from("Failed to parse number"))
	}
}

fn consume_while<F>(it: &mut Peekable<Chars>, x: F) -> Vec<char>
	where F : Fn(char) -> bool {

	let mut v: Vec<char> = vec![];

	while let Some(&ch) = it.peek() {
		if x(ch) {
			it.next().unwrap();
			v.push(ch);
		} else {
			break;
		}
	}

	v
}
