use super::parser::{Expr, Op};

pub fn evaluate(expression: Expr) -> Result<Expr, String> {
	if let Expr::BinaryExpr(lhs, op, rhs) = expression {
		let lhs = evaluate(*lhs)?;
		let rhs = evaluate(*rhs)?;

		if let Expr::Integer(ref lhs_i) = lhs {
			if let Expr::Integer(ref rhs_i) = rhs {
				return Ok(match op {
					Op::Add => Expr::Integer(lhs_i + rhs_i),
					Op::Subtract => Expr::Integer(lhs_i - rhs_i),
					Op::Multiply => Expr::Integer(lhs_i * rhs_i),
					Op::Divide => Expr::Integer(lhs_i / rhs_i),
				});
			}
		}

		return Ok(Expr::BinaryExpr(Box::new(lhs), op, Box::new(rhs)));
	}

	Ok(expression)
}
