use super::tokenizer::{Token};

use std::fmt;
use std::iter::{Iterator, Peekable};

use num::{Zero, Signed, Integer, BigRational};
use num::bigint::{Sign, ToBigInt};

#[derive(Debug, PartialEq, Clone)]
pub enum Expr {
	Number(BigRational),
	Name(String),
	Boolean(bool),
	Tuple(Vec<Expr>),
	Assign(Box<Expr>, Box<Expr>),
	BinaryExpr(Box<Expr>, Op, Box<Expr>)
}

#[derive(Debug, PartialEq, Clone, Copy)]
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

fn op_prec(op: Op) -> u64 {
    use self::Op::*;
    match op {
        Add => 2,
        Subtract => 1,
        Multiply => 4,
        Adjacent => 4,
        Divide => 3,
        Modulus => 4,
        Exponent => 5,
        Equals => 6
    }
}

fn needs_paren(e: &Expr, p: Option<Op>) -> bool {
    let p = match p {
        Some(p) => p,
        None => return false
    };

    match *e {
        // needed to avoid confusing (a (-1)) with (a - 1)
        Expr::Number(ref i) => i.is_negative() && p == Op::Adjacent,
        Expr::BinaryExpr(_, ref o, _) => op_prec(*o) < op_prec(p),
        _ => false
    }
}

fn display_expr_parent(e: &Expr, f: &mut fmt::Formatter, p: Option<Op>) -> fmt::Result {
    use std::fmt::Display;

    let needs_paren = needs_paren(e, p);
    // println!("needs_paren {:?} => {}", e, needs_paren);

    if needs_paren {
        write!(f, "(")?;
    }

    match e {
        &Expr::Number(ref i) => {
            if f.alternate() {
                i.fmt(f)
            } else {
                fmt_ratio_decimal(i, f)
            }
        },
        &Expr::Name(ref n) => write!(f, "{}", n),
        &Expr::Boolean(ref b) => write!(f, "{}", b),
        &Expr::Tuple(ref v) => {
            write!(f, "(")?;

            let mut it = v.iter().peekable();

            if let Some(e) = it.next() {
                e.fmt(f)?;

                if it.peek().is_none() {
                    write!(f, ",")?;
                }
            }

            for e in it {
                write!(f, ",")?;
                e.fmt(f)?;
            }

            write!(f, ")")
        },
        &Expr::BinaryExpr(ref lhs, Op::Adjacent, ref rhs) => {
            write!(f, "(")?;
            lhs.fmt(f)?;
            write!(f, " ")?;
            rhs.fmt(f)?;
            write!(f, ")")
        },
        &Expr::Assign(ref lhs, ref rhs) => {
            write!(f, "(")?;
            lhs.fmt(f)?;
            write!(f, " ≔ ")?;
            rhs.fmt(f)?;
            write!(f, ")")
        },
        &Expr::BinaryExpr(ref lhs, ref op, ref rhs) => {
            display_expr_parent(lhs.as_ref(), f, Some(*op))?;
            write!(f, " ")?;
            op.fmt(f)?;
            write!(f, " ")?;
            display_expr_parent(rhs.as_ref(), f, Some(*op))
        }
    }?;

    if needs_paren {
        write!(f, ")")
    } else {
        Ok(())
    }
}

impl fmt::Display for Expr {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        display_expr_parent(self, f, None)
	}
}

fn fmt_ratio_decimal(r: &BigRational, f: &mut fmt::Formatter) -> fmt::Result {
	let precision = f.precision().unwrap_or(5);
	let base = 10.to_bigint().unwrap();

	let num = r.numer();
	let den = r.denom();
	if num.sign() == Sign::Minus {
		write!(f, "-")?;
	}

	let mut div = num.abs().div_rem(den);
	write!(f, "{}", div.0)?;
	if !div.1.is_zero() {
		write!(f, ".")?;
	}

	for _ in 0..precision {
		if div.1.is_zero() { break }
		div = (&base * div.1).div_rem(den);
		write!(f, "{}", div.0)?;
	}

	if !div.1.is_zero() {
		write!(f, "⋯")
	} else {
		Ok(())
	}
}

impl fmt::Display for Op {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			&Op::Add => write!(f, "+"),
			&Op::Subtract => write!(f, "−"),
			&Op::Multiply => write!(f, "∙"),
			&Op::Adjacent => write!(f, "∙"),
			&Op::Divide => write!(f, "÷"),
			&Op::Modulus => write!(f, "%"),
			&Op::Exponent => write!(f, "^"),
			&Op::Equals => write!(f, "="),
		}
	}
}

const UNARY_PRIORITY: u8 = 8;

fn get_precedence(token: &Token) -> Option<(u8, u8)> {
	match token {
		&Token::Equals => Some((3, 3)),
		&Token::Add | &Token::Subtract => Some((6, 6)),
		&Token::Multiply | &Token::Divide | &Token::Modulus => Some((7, 7)),
		&Token::Exponent => Some((10, 9)),
		_ => None
	}
}

pub fn parse(tokens: Vec<Token>) -> Result<Expr, String> {
	let mut it = tokens.iter().peekable();

	if it.peek().is_none() {
		return Ok(Expr::Tuple(vec![]));
	}

	let lhs = parse_expr(&mut it, 0)?;
	let next = it.next();

	if let Some(t) = next {
		if let &Token::Assign = t {
			let rhs = parse_expr(&mut it, 0)?;
			Ok(Expr::Assign(Box::new(lhs), Box::new(rhs)))
		} else {
			Err(format!("Unexpected token: {:?}", t))
		}
	} else {
		Ok(lhs)
	}
}

fn parse_expr<'a, It>(it: &mut Peekable<It>, precedence: u8) -> Result<Expr, String>
	where It: Iterator<Item=&'a Token> {

	let mut expr = parse_prefix(it)?;

	while let Some(&next_token) = it.peek() {
		match next_token {
			&Token::RightParen => break,
			&Token::Name(_) | &Token::Number(_) | &Token::LeftParen => {
				expr = Expr::BinaryExpr(
					Box::new(expr),
					Op::Adjacent,
					Box::new(parse_expr(it, 0)?) // FIXME: Is 0 the right precedence for this?
				);
				continue; // FIXME: Continue? Shouldn't this consume everything possible?
			},
			_ => {}
		}

		let (left_prec, right_prec) = match get_precedence(next_token) {
			Some((l, r)) => (l, r),
			None => break
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

	match it.next() {
		Some(t) => match t {
			&Token::Number(ref n) => {
				Ok(Expr::Number(n.clone()))
			},
			&Token::Name(ref n) => {
				Ok(Expr::Name(n.clone()))
			},
			&Token::Subtract => {
				Ok(Expr::BinaryExpr(
					Box::new(Expr::Number(BigRational::zero())),
					Op::Subtract,
					Box::new(parse_expr(it, UNARY_PRIORITY)?)))
			},
			&Token::LeftParen => {
				let mut exprs: Vec<Expr> = vec![];

				if let Some(&&Token::RightParen) = it.peek() {
					it.next().unwrap();
					return Ok(Expr::Tuple(exprs));
				}

				loop {
					exprs.push(parse_expr(it, 0)?);

					match it.next() {
						Some(&Token::RightParen) => break,
						Some(&Token::Comma) => {},
						_ => return Err(String::from("Missing right parenthesis"))
					}
				}

				if exprs.len() == 1 {
					Ok(exprs.swap_remove(0))
				} else {
					Ok(Expr::Tuple(exprs))
				}
			},
			_ => Err(format!("Unexpected token: {:?}", t))
		},
		None => Err(String::from("Unterminated expression"))
	}
}

fn parse_infix<'a, It>(left: Expr, it: &mut Peekable<It>, precedence: u8) -> Result<Expr, String>
	where It: Iterator<Item=&'a Token> {

	match it.next() {
		Some(t) => {
			let op = match t {
				&Token::Add => Op::Add,
				&Token::Subtract => Op::Subtract,
				&Token::Multiply => Op::Multiply,
				&Token::Divide => Op::Divide,
				&Token::Modulus => Op::Modulus,
				&Token::Exponent => Op::Exponent,
				&Token::Equals => Op::Equals,
				_ => return Err(format!("Unexpected token: {:?}", t))
			};

			let right = parse_expr(it, precedence)?;

			Ok(Expr::BinaryExpr(
				Box::new(left),
				op,
				Box::new(right)))
		},
		None => Err(String::from("No more tokens"))
	}
}
