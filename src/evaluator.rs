use super::parser::{Expr, Op};
use num::{One, Signed, ToPrimitive, FromPrimitive, BigRational};
use num::rational::Ratio;
use num::bigint::ToBigInt;

use super::context::Context;

use std::collections::HashSet;
use std::mem::swap;

pub fn evaluate(expression: Expr, context: &mut Context) -> Result<Expr, String> {
    let expression = normalize(&expression);

    Ok(expression)
}

fn normalize(expr: &Expr) -> Expr {
    let neg = BigRational::from_integer(FromPrimitive::from_i64(-1).unwrap());

    match expr {
        &Expr::BinaryExpr(ref lhs, ref op, ref rhs) => {
            let mut lhs = normalize(lhs);
            let mut op = op.clone();
            let mut rhs = normalize(rhs);

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

fn apply_associative(op: Op, lhs: Expr, rhs: Expr) -> (Expr, Expr) {
    match rhs {
        Expr::BinaryExpr(ref b, ref inner_op, ref c) if inner_op == &op => {
            let (a, b) = apply_associative(op, lhs, b.as_ref().clone());
            (Expr::BinaryExpr(Box::new(a), op, Box::new(b)), c.as_ref().clone())
        },
        _ => (lhs, rhs)
    }
}

fn reductions(expr: &Expr, context: &mut Context) -> Vec<Expr> {
    let mut vec = Vec::new();

    // (a + b) -> [a + b]
    //     where a: Number
    //     where b: Number
    if let &Expr::BinaryExpr(ref lhs, Op::Add, ref rhs) = expr {
        if let &Expr::Number(ref a) = &**lhs {
            if let &Expr::Number(ref b) = &**rhs {
                vec.push(Expr::Number(a + b));
            }
        }
    }

    // (a * b) -> [a * b]
    //     where a: Number
    //     where b: Number
    if let &Expr::BinaryExpr(ref lhs, Op::Multiply, ref rhs) = expr {
        if let &Expr::Number(ref a) = &**lhs {
            if let &Expr::Number(ref b) = &**rhs {
                vec.push(Expr::Number(a * b));
            }
        }
    }

    // (a + b) -> (b + a)
    if let &Expr::BinaryExpr(ref lhs, Op::Add, ref rhs) = expr {
        vec.push(Expr::BinaryExpr(
            rhs.clone(),
            Op::Add,
            lhs.clone()));
    }

    // a + (b + c) -> (a + b) + c
    if let &Expr::BinaryExpr(ref a, Op::Add, ref rhs) = expr {
        if let &Expr::BinaryExpr(ref b, Op::Add, ref c) = &**rhs {
            vec.push(Expr::BinaryExpr(
                Box::new(Expr::BinaryExpr(
                    a.clone(),
                    Op::Add,
                    b.clone()
                )),
                Op::Add,
                c.clone()));
        }
    }

    // a * (b * c) -> (a * b) * c
    if let &Expr::BinaryExpr(ref a, Op::Multiply, ref rhs) = expr {
        if let &Expr::BinaryExpr(ref b, Op::Multiply, ref c) = &**rhs {
            vec.push(Expr::BinaryExpr(
                Box::new(Expr::BinaryExpr(
                    a.clone(),
                    Op::Multiply,
                    b.clone()
                )),
                Op::Multiply,
                c.clone()));
        }
    }

    // (a * x) + (b * x) -> (a + b) * x
    if let &Expr::BinaryExpr(ref lhs, Op::Add, ref rhs) = expr {
        if let &Expr::BinaryExpr(ref a, Op::Multiply, ref x1) = &**lhs {
            if let &Expr::BinaryExpr(ref b, Op::Multiply, ref x2) = &**rhs {
                if x1 == x2 {
                    vec.push(Expr::BinaryExpr(
                        Box::new(Expr::BinaryExpr(
                            a.clone(),
                            Op::Add,
                            b.clone()
                        )),
                        Op::Multiply,
                        x1.clone()));
                }
            }
        }
    }

    // (a x) + (b x) -> (a + b) x
    if let &Expr::BinaryExpr(ref lhs, Op::Add, ref rhs) = expr {
        if let &Expr::BinaryExpr(ref a, Op::Adjacent, ref x1) = &**lhs {
            if let &Expr::BinaryExpr(ref b, Op::Adjacent, ref x2) = &**rhs {
                if x1 == x2 {
                    vec.push(Expr::BinaryExpr(
                        Box::new(Expr::BinaryExpr(
                            a.clone(),
                            Op::Add,
                            b.clone()
                        )),
                        Op::Adjacent,
                        x1.clone()));
                }
            }
        }
    }

    // (a * x) + x -> ((a + 1) * x)
    if let &Expr::BinaryExpr(ref lhs, Op::Add, ref x1) = expr {
        if let &Expr::BinaryExpr(ref a, Op::Multiply, ref x2) = &**lhs {
            if x1 == x2 {
                vec.push(Expr::BinaryExpr(
                    Box::new(Expr::BinaryExpr(
                        a.clone(),
                        Op::Add,
                        Box::new(Expr::Number(BigRational::one()))
                    )),
                    Op::Multiply,
                    x1.clone()));
            }
        }
    }

    // (a x) + x -> ((a + 1) x)
    if let &Expr::BinaryExpr(ref lhs, Op::Add, ref x1) = expr {
        if let &Expr::BinaryExpr(ref a, Op::Adjacent, ref x2) = &**lhs {
            if x1 == x2 {
                vec.push(Expr::BinaryExpr(
                    Box::new(Expr::BinaryExpr(
                        a.clone(),
                        Op::Add,
                        Box::new(Expr::Number(BigRational::one()))
                    )),
                    Op::Adjacent,
                    x1.clone()));
            }
        }
    }

    // Explore subexpressions
    if let &Expr::BinaryExpr(ref lhs, ref op, ref rhs) = expr {
        for next in reductions(lhs, context).into_iter() {
            vec.push(Expr::BinaryExpr(
                Box::new(next),
                op.clone(),
                rhs.clone()));
        }
        for next in reductions(rhs, context).into_iter() {
            vec.push(Expr::BinaryExpr(
                lhs.clone(),
                op.clone(),
                Box::new(next)));
        }
    }

    vec
}
