use super::parser::{Expr, Op};
use num::{ToPrimitive, BigRational};
use num::rational::Ratio;
use num::bigint::ToBigInt;

pub fn evaluate(expression: Expr) -> Result<Expr, String> {
	if let Expr::BinaryExpr(lhs, op, rhs) = expression {
		let lhs = evaluate(*lhs)?;
		let rhs = evaluate(*rhs)?;

		if let Expr::Number(ref lhs_i) = lhs {
			if let Expr::Number(ref rhs_i) = rhs {
				match op {
					Op::Add => return Ok(Expr::Number(lhs_i + rhs_i)),
					Op::Subtract => return Ok(Expr::Number(lhs_i - rhs_i)),
					Op::Multiply => return Ok(Expr::Number(lhs_i * rhs_i)),
					Op::Adjacent => return Ok(Expr::Number(lhs_i * rhs_i)),
					Op::Divide => return Ok(Expr::Number(lhs_i / rhs_i)),
					Op::Modulus => return Ok(Expr::Number(lhs_i % rhs_i)),
					Op::Exponent => {
						if let Some(r) = ratio_power(lhs_i, rhs_i) {
							return Ok(Expr::Number(r));
						}
					},
					Op::Equals => return Ok(Expr::Boolean(lhs_i == rhs_i))
				}
			}
		}

		return Ok(Expr::BinaryExpr(Box::new(lhs), op, Box::new(rhs)));
	}

	Ok(expression)
}

fn ratio_power(lhs: &BigRational, rhs: &BigRational) -> Option<BigRational> {
	if !rhs.is_integer() {
		println!("Note: Non-integer exponents ({}) are not supported", rhs);
		return None;
	}

	let power = rhs.numer().to_i32();
	let numer = lhs.numer().to_isize();
	let denom = lhs.denom().to_isize();

	if let (Some(p), Some(n), Some(d)) = (power, numer, denom) {
		let r = Ratio::new(n, d).pow(p);
		let numer = r.numer().to_bigint();
		let denom = r.denom().to_bigint();

		if let (Some(n), Some(d)) = (numer, denom) {
			return Some(BigRational::new(n, d));
		}
	}
	
	None
}
