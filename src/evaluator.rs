use super::parser::{Expr, Op};

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
					Op::Exponent => println!("NYI: a^b"),
					Op::Equals => return Ok(Expr::Boolean(lhs_i == rhs_i))
				}
			}
		}

		return Ok(Expr::BinaryExpr(Box::new(lhs), op, Box::new(rhs)));
	}

	Ok(expression)
}
