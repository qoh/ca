use super::parser::{Expr, Op};
use num::{pow, Zero, One, Signed, ToPrimitive, FromPrimitive, BigRational};
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
                // TODO: Turn (function a) into Application(function, a)
                // TODO: Turn (a unit) into Measure(a, unit)
                op = Op::Multiply;
            }

            if op == Op::Subtract {
                // (a - b) => (a + (-1 * b))
                /*let zero = if let Expr::Number(ref a) = lhs {
                    a.is_zero()
                } else {
                    false
                };

                if zero {
                    lhs = Expr::Number(neg);
                    op = Op::Multiply;
                } else {*/
                    op = Op::Add;
                    rhs = Expr::BinaryExpr(
                        Box::new(Expr::Number(neg)),
                        Op::Multiply,
                        Box::new(rhs));
                //}
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

    if expr != &new_expr  {
        // println!("simplify {} => {}", expr, &new_expr);
    }

    new_expr
}

fn simplify_inner(expr: &Expr) -> Expr {
    match *expr {
        Expr::BinaryExpr(_, Op::Add, _) => simplify_add(expr),
        Expr::BinaryExpr(_, Op::Multiply, _) => simplify_multiply(expr),
        Expr::BinaryExpr(ref a, Op::Exponent, ref b) => {
            let a = simplify(a);
            let b = simplify(b);

            if let Expr::Number(ref a) = a {
                if let Expr::Number(ref b) = b {
                    if let Some(n) = real_power(a, b) {
                        return Expr::Number(n);
                    }
                }
            }

            Expr::BinaryExpr(Box::new(a), Op::Exponent, Box::new(b))
        },
        _ => expr.clone()
    }
}

fn real_power(a: &BigRational, b: &BigRational) -> Option<BigRational> {
    if !b.is_integer() {
        return None;
    }

    // a ^ -1
    if b.numer() == &-1.to_bigint().unwrap() {
        return Some(BigRational::new(
            a.denom().clone(),
            a.numer().clone()));
    }

    // ( 1/1) ^ any
    // (-1/1) ^ any
    if a.is_integer() && (a.numer().abs() == 1.to_bigint().unwrap()) {
        return Some(a.clone());
    }

    // (a/1) ^ usize
    if a.denom() == &1.to_bigint().unwrap() {
        if let Some(exp) = b.numer().to_usize() {
            let base = a.numer().clone();
            return Some(BigRational::new(pow(base, exp), 1.to_bigint().unwrap()));
        }
    }

    // (isize/isize) ^ i32
    if let (Some(p), Some(n), Some(d)) = (b.numer().to_i32(),
                                          a.numer().to_isize(),
                                          b.denom().to_isize()) {
        let r = Ratio::new(n, d).pow(p);
        let n = r.numer().to_bigint().unwrap();
        let d = r.denom().to_bigint().unwrap();
        return Some(BigRational::new(n, d));
    }

    None
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

    for (item, coeff) in items.iter().zip(coefficients) {
        if let Expr::Number(ref n) = *item {
            sum = sum + coeff * n;
        } else if coeff == BigRational::one() {
            replacement.push(item.clone());
        } else if coeff != BigRational::zero() {
            replacement.push(Expr::BinaryExpr(
                Box::new(Expr::Number(coeff.clone())),
                Op::Multiply, Box::new(item.clone())));
        }
    }

    if sum != BigRational::zero() {
        replacement.insert(0, Expr::Number(sum));
    }

    let mut result = match replacement.pop() {
        Some(e) => e,
        None => Expr::Number(BigRational::zero())
    };

    while let Some(next_result) = replacement.pop() {
        result = Expr::BinaryExpr(Box::new(next_result),
            Op::Add, Box::new(result));
    }

    result
}

fn simplify_multiply(expr: &Expr) -> Expr {
    #[derive(Clone)]
    struct Term {
        coeff: BigRational,
        base: Expr,
        power: BigRational
    }

    let mut items = Vec::new();
    let mut current = expr.clone();

    while let Expr::BinaryExpr(lhs, Op::Multiply, rhs) = current {
        items.push(*lhs);
        current = *rhs;
    }

    items.push(current);

    let mut terms: Vec<Term> = items.iter().map(|i| Term {
        coeff: BigRational::one(),
        base: simplify(i),
        power: BigRational::one()}).collect();
    let mut coeff = BigRational::one();

    for term in &mut terms {
        let mut new = term.clone();

        if let Expr::Number(ref n) = term.base {
            coeff = &coeff * n;
            new.coeff = BigRational::zero();
        } else if let Expr::BinaryExpr(ref a, Op::Exponent, ref b) = term.base {
            if let Expr::Number(ref n) = **b {
                new.power = n.clone();
                new.base = (**a).clone();
            }
        }

        *term = new;
    }

    for i in 0..terms.len() {
        if !terms[i].coeff.is_zero() {
            for j in i+1..terms.len() {
                if terms[i].base == terms[j].base {
                    terms[i].coeff = &terms[i].coeff * &terms[j].coeff;
                    terms[i].power = &terms[i].power + &terms[j].power;
                    terms[j].coeff = BigRational::zero();
                }
            }
        }
    }

    let mut replacement = Vec::new();

    for term in terms {
        if !term.power.is_zero() {
            let mut e = term.base.clone();

            if term.power != BigRational::one() {
                e = Expr::BinaryExpr(Box::new(e), Op::Exponent,
                    Box::new(Expr::Number(term.power.clone())));
            }

            if term.coeff == BigRational::one() {
                replacement.push(e);
            } else if !term.coeff.is_zero() {
                replacement.push(Expr::BinaryExpr(
                    Box::new(Expr::Number(term.coeff.clone())),
                    Op::Multiply, Box::new(e)));
            }
        }
    }

    if coeff == BigRational::zero() {
        return Expr::Number(coeff);
    }

    if coeff != BigRational::one() {
        replacement.insert(0, Expr::Number(coeff));
    }

    let mut result = match replacement.pop() {
        Some(e) => e,
        None => Expr::Number(BigRational::one())
    };

    while let Some(next_result) = replacement.pop() {
        result = Expr::BinaryExpr(Box::new(next_result),
            Op::Multiply, Box::new(result));
    }

    result
}
