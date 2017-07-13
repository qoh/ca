use super::parser::{Expr, Op};
use num::{Zero, One, Signed, ToPrimitive, FromPrimitive, BigRational};
use num::rational::Ratio;
use num::bigint::ToBigInt;

use super::context::Context;

use std::collections::HashSet;
use std::mem::swap;

pub fn evaluate(expression: Expr, context: &mut Context) -> Result<Expr, String> {
    let expression = normalize(&expression);
    let expression = simplify(&expression);

    Ok(expression)
}

fn normalize(expr: &Expr) -> Expr {
    let neg = BigRational::from_integer(FromPrimitive::from_i64(-1).unwrap());

    match expr {
        &Expr::BinaryExpr(ref lhs, op, ref rhs) => {
            let mut lhs = normalize(lhs);
            let mut op = op;
            let mut rhs = normalize(rhs);

            if op == Op::Adjacent {
                op = Op::Multiply;
            }

            if op == Op::Subtract {
                // (a - b) => (a + (-1 * b))
                op = Op::Add;
                rhs = Expr::BinaryExpr(
                    Box::new(Expr::Number(neg)),
                    Op::Multiply,
                    Box::new(rhs));
            } else if op == Op::Divide {
                // (a / b) => (a * (b ^ -1))
                op = Op::Multiply;
                rhs = Expr::BinaryExpr(
                    Box::new(rhs),
                    Op::Exponent,
                    Box::new(Expr::Number(neg)));
            }

            if op == Op::Multiply || op == Op::Add {
                let (new_lhs, new_rhs) = apply_associative(op, lhs, rhs);
                lhs = new_lhs;
                rhs = new_rhs;
            }

            Expr::BinaryExpr(Box::new(lhs), op, Box::new(rhs))
        },
        e => e.clone()
    }
}

/*fn apply_associative(op: Op, lhs: Expr, rhs: Expr) -> (Expr, Expr) {
    match rhs {
        Expr::BinaryExpr(ref b, ref inner_op, ref c) if inner_op == &op => {
            let (a, b) = apply_associative(op, lhs, b.as_ref().clone());
            (Expr::BinaryExpr(Box::new(a), op, Box::new(b)), c.as_ref().clone())
        },
        _ => (lhs, rhs)
    }
}*/
fn apply_associative(op: Op, lhs: Expr, rhs: Expr) -> (Expr, Expr) {
    match lhs {
        Expr::BinaryExpr(ref a, ref inner_op, ref b) if inner_op == &op => {
            let (b, rhs) = apply_associative(op, b.as_ref().clone(), rhs);
            (a.as_ref().clone(), Expr::BinaryExpr(Box::new(b), op, Box::new(rhs)))
        },
        _ => (lhs, rhs)
    }
}

// Assumes that `expr` has already been normalized via `normalize()`
fn simplify(expr: &Expr) -> Expr {
    let new_expr = simplify_inner(expr);
    println!("simplify {} => {}", expr, &new_expr);
    new_expr
}

fn simplify_inner(expr: &Expr) -> Expr {
    match expr {
        &Expr::BinaryExpr(_, Op::Add, _) => simplify_add(expr),
        &Expr::BinaryExpr(_, Op::Multiply, _) => simplify_multiply(expr),
        _ => expr.clone()
    }
}

fn simplify_add(expr: &Expr) -> Expr {
    let mut items = Vec::new();
    let mut current = expr.clone();

    while let Expr::BinaryExpr(lhs, Op::Add, rhs) = current {
        items.push(*lhs);
        current = *rhs;
    }

    items.push(current);

    let mut coefficients = vec![BigRational::from_integer(FromPrimitive::from_u64(1).unwrap()); items.len()];

    for i in 0..items.len() {
        let mut new = simplify(&items[i]);

        new = if let Expr::BinaryExpr(ref l, Op::Multiply, ref f) = new {
            if let Expr::Number(ref c) = **l {
                coefficients[i] = c.clone();
                f.as_ref().clone()
            } else {
                new.clone()
            }
        } else {
            new.clone()
        };

        items[i] = new;
    }

    for i in 0..items.len() {
        if !coefficients[i].is_zero() {
            for j in i+1..items.len() {
                if items[i] == items[j] {
                    coefficients[i] = &coefficients[i] + &coefficients[j];
                    coefficients[j] = BigRational::zero();
                }
            }
        }
    }

    let mut replacement = Vec::new();
    let mut sum = BigRational::zero();

    for i in 0..coefficients.len() {
        if let Expr::Number(ref n) = items[i] {
            sum = sum + &coefficients[i] * n;
        } else if coefficients[i] == BigRational::one() {
            replacement.push((&items[i]).clone());
        } else if coefficients[i] != BigRational::zero() {
            replacement.push(Expr::BinaryExpr(
                Box::new(Expr::Number((&coefficients[i]).clone())),
                Op::Multiply,
                Box::new((&items[i]).clone())
            ));
        }
    }

    if sum != BigRational::zero() {
        replacement.insert(0, Expr::Number(sum));
    }

    let mut result = replacement.pop().unwrap(); // bad unwrap

    while let Some(next_result) = replacement.pop() {
        result = Expr::BinaryExpr(
            Box::new(next_result),
            Op::Add,
            Box::new(result));
    }

    result
}

fn simplify_multiply(expr: &Expr) -> Expr {
    let mut items = Vec::new();
    let mut current = expr.clone();

    while let Expr::BinaryExpr(lhs, Op::Multiply, rhs) = current {
        items.push(*lhs);
        current = *rhs;
    }

    items.push(current);

    let mut terms: Vec<(BigRational, Expr, BigRational)> = items.iter().map(|i| (BigRational::one(), simplify(i), BigRational::one())).collect();
    let mut coeff = BigRational::one();

    for i in 0..terms.len() {
        let mut new = (&terms[i]).clone();

        if let Expr::Number(ref n) = terms[i].1 {
            coeff = &coeff * n;
            new.0 = BigRational::zero();
        } else if let Expr::BinaryExpr(_, Op::Multiply, _) = terms[i].1 {
            panic!("multiply");
        } else if let Expr::BinaryExpr(_, Op::Exponent, _) = terms[i].1 {
            panic!("exponent");
        }
        // shouldn't be a need to handle multiply, but do so here
        // handle exponent here

        terms[i] = new;
    }

    for i in 0..terms.len() {
        if !(&terms[i]).0.is_zero() {
            for j in i+1..terms.len() {
                if (&terms[i]).1 == (&terms[j]).1 {
                    terms[i].0 = &terms[i].0 * &terms[j].0;
                    terms[i].2 = &terms[i].2 + &terms[j].2;
                    terms[j].0 = BigRational::zero();
                }
            }
        }
    }

    let mut replacement = Vec::new();

    for i in 0..terms.len() {
        if !(&terms[i]).2.is_zero() {
            let mut e = (&terms[i]).1.clone();

            if (&terms[i]).2 != BigRational::one() {
                e = Expr::BinaryExpr(Box::new(e), Op::Exponent, Box::new(Expr::Number((&terms[i]).2.clone())));
            }

            if (&terms[i]).0 == BigRational::one() {
                replacement.push(e);
            } else if !(&terms[i]).0.is_zero() {
                replacement.push(Expr::BinaryExpr(
                    Box::new(Expr::Number((&terms[i]).0.clone())),
                    Op::Multiply,
                    Box::new(e)));
            }
        }
    }

    if coeff != BigRational::one() {
        replacement.insert(0, Expr::Number(coeff));
    }

    let mut result = replacement.pop().unwrap(); // bad unwrap

    while let Some(next_result) = replacement.pop() {
        result = Expr::BinaryExpr(
            Box::new(next_result),
            Op::Multiply,
            Box::new(result));
    }

    result
}
