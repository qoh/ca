use super::parser::{Expr, Op};

pub fn evaluate(expression: Expr) -> Result<Expr, String> {
	if let Expr::BinaryExpr(lhs, op, rhs) = expression {
		let lhs = evaluate(*lhs)?;
		let rhs = evaluate(*rhs)?;

		if let Expr::Number(ref lhs_i) = lhs {
			if let Expr::Number(ref rhs_i) = rhs {
				return Ok(match op {
					Op::Add => Expr::Number(lhs_i + rhs_i),
					Op::Subtract => Expr::Number(lhs_i - rhs_i),
					Op::Multiply => Expr::Number(lhs_i * rhs_i),
					Op::Divide => Expr::Number(lhs_i / rhs_i),
				});
			}
		}

		return Ok(Expr::BinaryExpr(Box::new(lhs), op, Box::new(rhs)));
	}

	Ok(expression)
}
