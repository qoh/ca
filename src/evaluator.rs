use super::parser::{Expr, Op};
use num::{One, Signed, ToPrimitive, BigRational};
use num::rational::Ratio;
use num::bigint::ToBigInt;

use super::context::Context;

use std::collections::HashSet;

pub fn evaluate(expression: Expr, context: &mut Context) -> Result<Expr, String> {
    let mut best_expr = expression.clone();
    let mut best_score = None;

    let mut seen = HashSet::new();
    let mut stack = Vec::new();

    seen.insert(expression.clone());
    stack.push(expression);

    loop {
        let current = match stack.pop() {
            Some(e) => e,
            None => break
        };

        let score = score(&current);

        if best_score == None || best_score.unwrap() > score {
            best_score = Some(score);
            best_expr = current.clone();
        }

        for next in reductions(&current, context).into_iter() {
            if seen.contains(&next) {
                continue;
            }

            seen.insert(next.clone());
            stack.push(next);
        }
    }

    // Ok(Expr::Tuple(vec![]))
    Ok(best_expr)
}

fn score(expr: &Expr) -> i64 {
    match expr {
        &Expr::Number(_) => 1,
        &Expr::Name(_) => 2,
        &Expr::Boolean(_) => 1,
        &Expr::Tuple(_) => 3,
        &Expr::Assign(_, _) => 4,
        &Expr::BinaryExpr(ref lhs, ref op, ref rhs) =>
            1 + score(&**lhs) + score(&**rhs),
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
