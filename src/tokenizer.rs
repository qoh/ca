use std::iter::Peekable;
use std::str::Chars;

use num::{BigInt, BigRational};
use num::bigint::ToBigInt;

#[derive(Debug, PartialEq)]
pub enum Symbol {
	Add,
	Subtract,
	Multiply,
	Divide,
	Modulus,
	Exponent,
	Equals
}

#[derive(Debug, PartialEq)]
pub enum Token {
	Integer(BigRational),
	Operator(Symbol)
}

pub trait Tokenizer {
	fn tokenize(&self) -> Vec<Token>;
}

impl Tokenizer for String {
	fn tokenize(&self) -> Vec<Token> {
		let mut it = self.chars().peekable();
		let mut tokens: Vec<Token> = vec![];

		loop {
			match it.peek() {
				Some(&ch) => match ch {
					'0' ... '9' => {
						let num: String = consume_while(&mut it, |a| a.is_numeric() || a == '_' || a == '.')
							.into_iter()
							.collect();
						tokens.push(Token::Integer(BigRational::new(num.parse::<BigInt>().unwrap(), 1.to_bigint().unwrap())));
					},
					'+' => {
						it.next().unwrap();
						tokens.push(Token::Operator(Symbol::Add));
					},
					'-' => {
						it.next().unwrap();
						tokens.push(Token::Operator(Symbol::Subtract));
					},
					'*' => {
						it.next().unwrap();
						tokens.push(Token::Operator(Symbol::Multiply));
					},
					'/' => {
						it.next().unwrap();
						tokens.push(Token::Operator(Symbol::Divide));
					},
					'%' => {
						it.next().unwrap();
						tokens.push(Token::Operator(Symbol::Modulus));
					},
					'^' => {
						it.next().unwrap();
						tokens.push(Token::Operator(Symbol::Exponent));
					},
					'=' => {
						it.next().unwrap();
						tokens.push(Token::Operator(Symbol::Equals));
					},
					'\n' => {
						it.next().unwrap();
					},
					_ => panic!("invalid char {}", ch)
				},
				None => break
			}
		}

		tokens
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
