//! Equation solving without a CAS. One unknown: analytic inversion when it occurs once, closed-form roots when
//! the residual is polynomial, otherwise a wide log scan + Brent with a residual check (rejects poles).
//! Systems: isolate an unknown that occurs once, substitute, recurse; Newton multistart as the fallback.
//! Results are sorted ascending (system solutions as tuples, ordered by the variables sorted by name).

use std::collections::HashMap;

use crate::expr::{Expr, Func, Op};

const REL_TOL: f64 = 1e-9;

fn n(x: f64) -> Expr {
    Expr::Num(x)
}

fn bin(op: Op, a: Expr, b: Expr) -> Expr {
    Expr::Bin(op, Box::new(a), Box::new(b))
}

fn constant(e: &Expr) -> Option<f64> {
    e.eval(&|_| None)
}

// ------------------------------------------------------------------ symbolic isolation

/// Expressions for `x` such that `e == target`, valid when `x` occurs exactly once in `e`.
/// Branches follow sympy: sin -> asin, pi - asin; cos -> +-acos; tan -> atan (principal); even powers -> +-.
pub fn isolate(e: &Expr, x: &str, target: Expr) -> Vec<Expr> {
    match e {
        Expr::Var(v) if v == x => vec![target],
        Expr::Neg(a) => isolate(a, x, Expr::Neg(Box::new(target))),
        Expr::Bin(op, a, b) => {
            let in_a = a.count(x) > 0;
            let (side, other) = if in_a {
                (a.as_ref(), b.as_ref().clone())
            } else {
                (b.as_ref(), a.as_ref().clone())
            };
            match (op, in_a) {
                (Op::Add, _) => isolate(side, x, bin(Op::Sub, target, other)),
                (Op::Sub, true) => isolate(side, x, bin(Op::Add, target, other)),
                (Op::Sub, false) => isolate(side, x, bin(Op::Sub, other, target)),
                (Op::Mul, _) => isolate(side, x, bin(Op::Div, target, other)),
                (Op::Div, true) => isolate(side, x, bin(Op::Mul, target, other)),
                (Op::Div, false) => isolate(side, x, bin(Op::Div, other, target)),
                (Op::Pow, true) => {
                    let p = constant(&other);
                    let root = bin(Op::Pow, target.clone(), bin(Op::Div, n(1.0), other.clone()));
                    match p {
                        Some(p) if p.fract() == 0.0 && (p as i64) % 2 == 0 => {
                            let mut out = isolate(side, x, Expr::Neg(Box::new(root.clone())));
                            out.extend(isolate(side, x, root));
                            out
                        }
                        Some(p) if p.fract() == 0.0 => {
                            // odd integer power: real root keeps the sign
                            let signed_root = bin(
                                Op::Mul,
                                Expr::Call(Func::Abs, Box::new(target.clone())),
                                n(1.0),
                            );
                            let mag = bin(Op::Pow, signed_root, bin(Op::Div, n(1.0), other));
                            let sign = bin(
                                Op::Div,
                                target.clone(),
                                Expr::Call(Func::Abs, Box::new(target)),
                            );
                            isolate(side, x, bin(Op::Mul, sign, mag))
                        }
                        _ => isolate(side, x, root),
                    }
                }
                (Op::Pow, false) => {
                    let ln = |t: Expr| Expr::Call(Func::Ln, Box::new(t));
                    isolate(side, x, bin(Op::Div, ln(target), ln(other)))
                }
            }
        }
        Expr::Call(f, a) => {
            let call = |g: Func, t: Expr| Expr::Call(g, Box::new(t));
            let targets = match f {
                Func::Exp => vec![call(Func::Ln, target)],
                Func::Ln => vec![call(Func::Exp, target)],
                Func::Sqrt => vec![bin(Op::Pow, target, n(2.0))],
                Func::Sin => vec![
                    call(Func::Asin, target.clone()),
                    bin(Op::Sub, n(std::f64::consts::PI), call(Func::Asin, target)),
                ],
                Func::Cos => vec![
                    call(Func::Acos, target.clone()),
                    bin(Op::Sub, n(std::f64::consts::TAU), call(Func::Acos, target)),
                ],
                Func::Tan => vec![call(Func::Atan, target)],
                Func::Asin => vec![call(Func::Sin, target)],
                Func::Acos => vec![call(Func::Cos, target)],
                Func::Atan => vec![call(Func::Tan, target)],
                Func::Abs => vec![Expr::Neg(Box::new(target.clone())), target],
            };
            targets.into_iter().flat_map(|t| isolate(a, x, t)).collect()
        }
        _ => vec![],
    }
}

// ------------------------------------------------------------------ polynomials

type Poly = Vec<f64>; // coefficients, index = power

fn padd(a: &Poly, b: &Poly, sign: f64) -> Poly {
    (0..a.len().max(b.len()))
        .map(|i| a.get(i).copied().unwrap_or(0.0) + sign * b.get(i).copied().unwrap_or(0.0))
        .collect()
}

fn pmul(a: &Poly, b: &Poly) -> Poly {
    let mut out = vec![0.0; a.len() + b.len() - 1];
    for (i, x) in a.iter().enumerate() {
        for (j, y) in b.iter().enumerate() {
            out[i + j] += x * y;
        }
    }
    out
}

/// The residual as a rational function num/den in `x`, if it is one (integer powers only).
fn rational(e: &Expr, x: &str) -> Option<(Poly, Poly)> {
    if e.count(x) == 0 {
        return Some((vec![constant(e)?], vec![1.0]));
    }
    Some(match e {
        Expr::Var(_) => (vec![0.0, 1.0], vec![1.0]),
        Expr::Neg(a) => {
            let (p, q) = rational(a, x)?;
            (p.iter().map(|c| -c).collect(), q)
        }
        Expr::Bin(op, a, b) => {
            let (pa, qa) = rational(a, x)?;
            match op {
                Op::Pow => {
                    let k = constant(b)?;
                    if k.fract() != 0.0 || k.abs() > 12.0 {
                        return None;
                    }
                    let (mut p, mut q) = (vec![1.0], vec![1.0]);
                    for _ in 0..k.abs() as usize {
                        p = pmul(&p, &pa);
                        q = pmul(&q, &qa);
                    }
                    if k < 0.0 {
                        (q, p)
                    } else {
                        (p, q)
                    }
                }
                _ => {
                    let (pb, qb) = rational(b, x)?;
                    match op {
                        Op::Add => (padd(&pmul(&pa, &qb), &pmul(&pb, &qa), 1.0), pmul(&qa, &qb)),
                        Op::Sub => (padd(&pmul(&pa, &qb), &pmul(&pb, &qa), -1.0), pmul(&qa, &qb)),
                        Op::Mul => (pmul(&pa, &pb), pmul(&qa, &qb)),
                        Op::Div => (pmul(&pa, &qb), pmul(&qa, &pb)),
                        Op::Pow => unreachable!(),
                    }
                }
            }
        }
        _ => return None,
    })
}

fn peval(p: &Poly, x: f64) -> f64 {
    p.iter().rev().fold(0.0, |acc, c| acc * x + c)
}

fn trim(mut p: Poly) -> Poly {
    let scale = p.iter().fold(0.0_f64, |m, c| m.max(c.abs()));
    while p.len() > 1 && p.last().is_some_and(|c| c.abs() <= 1e-14 * scale) {
        p.pop();
    }
    p
}

fn poly_roots(p: &Poly) -> Vec<f64> {
    let p = trim(p.clone());
    let lead = *p.last().unwrap();
    let r: Vec<f64> = match p.len() {
        0 | 1 => vec![],
        2 => roots::find_roots_linear(p[1], p[0]).as_ref().to_vec(),
        3 => roots::find_roots_quadratic(p[2], p[1], p[0])
            .as_ref()
            .to_vec(),
        4 => roots::find_roots_cubic(p[3], p[2], p[1], p[0])
            .as_ref()
            .to_vec(),
        5 => roots::find_roots_quartic(p[4], p[3], p[2], p[1], p[0])
            .as_ref()
            .to_vec(),
        _ => aberth(&p.iter().map(|c| c / lead).collect::<Vec<_>>()),
    };
    // polish against the original polynomial
    let dp: Poly = p
        .iter()
        .enumerate()
        .skip(1)
        .map(|(i, c)| c * i as f64)
        .collect();
    r.into_iter()
        .map(|mut x| {
            for _ in 0..3 {
                let d = peval(&dp, x);
                if d != 0.0 {
                    let step = peval(&p, x) / d;
                    if step.is_finite() {
                        x -= step;
                    }
                }
            }
            x
        })
        .collect()
}

/// Real roots of a monic polynomial of degree > 4 (Aberth iteration over complex numbers).
fn aberth(p: &Poly) -> Vec<f64> {
    let deg = p.len() - 1;
    let radius = 1.0 + p[..deg].iter().fold(0.0_f64, |m, c| m.max(c.abs()));
    let mut z: Vec<(f64, f64)> = (0..deg)
        .map(|k| {
            let a = std::f64::consts::TAU * k as f64 / deg as f64 + 0.4;
            (radius * a.cos(), radius * a.sin())
        })
        .collect();
    let cmul = |a: (f64, f64), b: (f64, f64)| (a.0 * b.0 - a.1 * b.1, a.0 * b.1 + a.1 * b.0);
    let cdiv = |a: (f64, f64), b: (f64, f64)| {
        let d = b.0 * b.0 + b.1 * b.1;
        ((a.0 * b.0 + a.1 * b.1) / d, (a.1 * b.0 - a.0 * b.1) / d)
    };
    for _ in 0..500 {
        let mut moved = 0.0_f64;
        for i in 0..deg {
            let (mut f, mut df) = ((0.0, 0.0), (0.0, 0.0));
            for c in p.iter().rev() {
                df = (cmul(df, z[i]).0 + f.0, cmul(df, z[i]).1 + f.1);
                f = (cmul(f, z[i]).0 + c, cmul(f, z[i]).1);
            }
            let ratio = cdiv(f, df);
            let mut s = (0.0, 0.0);
            for j in (0..deg).filter(|&j| j != i) {
                let inv = cdiv((1.0, 0.0), (z[i].0 - z[j].0, z[i].1 - z[j].1));
                s = (s.0 + inv.0, s.1 + inv.1);
            }
            let denom = (1.0 - cmul(ratio, s).0, -cmul(ratio, s).1);
            let w = cdiv(ratio, denom);
            z[i] = (z[i].0 - w.0, z[i].1 - w.1);
            moved = moved.max((w.0 * w.0 + w.1 * w.1).sqrt());
        }
        if moved < 1e-15 * radius {
            break;
        }
    }
    z.into_iter()
        .filter(|c| c.1.abs() <= 1e-7 * c.0.abs().max(1.0))
        .map(|c| c.0)
        .collect()
}

// ------------------------------------------------------------------ one unknown

/// Magnitude bound of `e`: the expression evaluated with every term made positive (|x|+|1| for x-1, products
/// of bounds, ...). A root is accepted when the residual is tiny against it; at a pole both blow up together.
fn abs_eval(e: &Expr, env: &dyn Fn(&str) -> Option<f64>) -> f64 {
    let plain = |e: &Expr| e.eval(env).unwrap_or(f64::NAN).abs();
    match e {
        Expr::Num(v) => v.abs(),
        Expr::Var(_) | Expr::Call(..) => plain(e),
        Expr::Neg(a) => abs_eval(a, env),
        Expr::Bin(op, a, b) => match op {
            Op::Add | Op::Sub => abs_eval(a, env) + abs_eval(b, env),
            Op::Mul => abs_eval(a, env) * abs_eval(b, env),
            Op::Div => abs_eval(a, env) / plain(b),
            Op::Pow => match constant(b) {
                Some(k) if k > 0.0 => abs_eval(a, env).powf(k),
                _ => plain(e),
            },
        },
    }
}

fn term_scale(e: &Expr, x: &str, at: f64) -> f64 {
    abs_eval(e, &|v| (v == x).then_some(at))
}

fn residual_ok(e: &Expr, x: &str, r: f64) -> bool {
    let f = e.eval(&|v| (v == x).then_some(r));
    match f {
        Some(f) if f.is_finite() && r.is_finite() => {
            f.abs() <= 1e-8 * term_scale(e, x, r).max(1e-300)
        }
        _ => false,
    }
}

fn brent(f: &dyn Fn(f64) -> f64, mut a: f64, mut b: f64) -> Option<f64> {
    let (mut fa, mut fb) = (f(a), f(b));
    if fa * fb > 0.0 {
        return None;
    }
    let (mut c, mut fc, mut d) = (a, fa, b - a);
    let mut e = d;
    for _ in 0..200 {
        if fb * fc > 0.0 {
            c = a;
            fc = fa;
            d = b - a;
            e = d;
        }
        if fc.abs() < fb.abs() {
            (a, b, c) = (b, c, b);
            (fa, fb, fc) = (fb, fc, fb);
        }
        let tol = 2.0 * f64::EPSILON * b.abs() + 1e-300;
        let m = (c - b) / 2.0;
        if m.abs() <= tol || fb == 0.0 {
            return Some(b);
        }
        if e.abs() >= tol && fa.abs() > fb.abs() {
            let s = fb / fa;
            let (mut p, mut q) = if a == c {
                (2.0 * m * s, 1.0 - s)
            } else {
                let (q0, r) = (fa / fc, fb / fc);
                (
                    s * (2.0 * m * q0 * (q0 - r) - (b - a) * (r - 1.0)),
                    (q0 - 1.0) * (r - 1.0) * (s - 1.0),
                )
            };
            if p > 0.0 {
                q = -q;
            } else {
                p = -p;
            }
            if 2.0 * p < (3.0 * m * q - (tol * q).abs()).min((e * q).abs()) {
                e = d;
                d = p / q;
            } else {
                d = m;
                e = d;
            }
        } else {
            d = m;
            e = d;
        }
        a = b;
        fa = fb;
        b += if d.abs() > tol { d } else { tol.copysign(m) };
        fb = f(b);
    }
    Some(b)
}

fn scan(e: &Expr, x: &str, positive: bool) -> Vec<f64> {
    let f = |v: f64| e.eval(&|name| (name == x).then_some(v)).unwrap_or(f64::NAN);
    let mut grid: Vec<f64> = (0..=1800)
        .map(|i| 10f64.powf(-15.0 + 30.0 * i as f64 / 1800.0))
        .collect();
    if !positive {
        let neg: Vec<f64> = grid.iter().rev().map(|v| -v).collect();
        grid = neg.into_iter().chain([0.0]).chain(grid).collect();
    } else {
        grid.insert(0, 0.0);
    }
    let vals: Vec<f64> = grid.iter().map(|&v| f(v)).collect();
    let mut out = Vec::new();
    for i in 0..grid.len() - 1 {
        let (fa, fb) = (vals[i], vals[i + 1]);
        if !fa.is_finite() || !fb.is_finite() {
            continue;
        }
        if fa == 0.0 {
            out.push(grid[i]);
        } else if fa * fb < 0.0 {
            if let Some(r) = brent(&f, grid[i], grid[i + 1]) {
                out.push(r);
            }
        } else if i > 0
            && vals[i - 1].is_finite()
            && fa.abs() < vals[i - 1].abs()
            && fa.abs() < fb.abs()
        {
            // local minimum of |f| without a sign change: a touching root? polish with Newton
            let mut r = grid[i];
            for _ in 0..60 {
                let h = 1e-7 * r.abs().max(1e-12);
                let d = (f(r + h) - f(r - h)) / (2.0 * h);
                if d == 0.0 || !d.is_finite() {
                    break;
                }
                r -= f(r) / d;
            }
            out.push(r);
        }
    }
    out
}

fn finalize(mut roots: Vec<f64>, e: &Expr, x: &str, positive: bool) -> Vec<f64> {
    roots.retain(|&r| r.is_finite() && (!positive || r >= 0.0) && residual_ok(e, x, r));
    roots.sort_by(f64::total_cmp);
    roots.dedup_by(|a, b| (*a - *b).abs() <= REL_TOL * a.abs().max(b.abs()).max(1e-300));
    roots
}

/// All real roots of `e(x) = 0` where `x` is the only free variable.
pub fn solve1(e: &Expr, x: &str, positive: bool) -> Vec<f64> {
    if e.count(x) == 1 {
        let cands: Vec<f64> = isolate(e, x, n(0.0)).iter().filter_map(constant).collect();
        let r = finalize(cands, e, x, positive);
        if !r.is_empty() {
            return r;
        }
    }
    if let Some((num, den)) = rational(e, x) {
        let cands: Vec<f64> = poly_roots(&num)
            .into_iter()
            .filter(|&r| peval(&den, r).abs() > 1e-300)
            .collect();
        return finalize(cands, e, x, positive);
    }
    finalize(scan(e, x, positive), e, x, positive)
}

// ------------------------------------------------------------------ systems

fn subst_var(e: &Expr, x: &str, with: &Expr) -> Expr {
    match e {
        Expr::Var(v) if v == x => with.clone(),
        Expr::Num(_) | Expr::Var(_) => e.clone(),
        Expr::Neg(a) => Expr::Neg(Box::new(subst_var(a, x, with))),
        Expr::Call(f, a) => Expr::Call(*f, Box::new(subst_var(a, x, with))),
        Expr::Bin(op, a, b) => bin(*op, subst_var(a, x, with), subst_var(b, x, with)),
    }
}

/// Solutions of `eqs = 0` in the unknowns `xs` (sorted by name), each as a tuple aligned with `xs`.
pub fn solve_system(eqs: &[Expr], xs: &[String], positive: &dyn Fn(&str) -> bool) -> Vec<Vec<f64>> {
    let mut xs: Vec<String> = xs.to_vec();
    xs.sort();
    let sols = eliminate(eqs, &xs, positive, 0);
    let mut sols: Vec<Vec<f64>> = if sols.is_empty() {
        newton(eqs, &xs, positive)
    } else {
        sols
    };
    sols.sort_by(|a, b| {
        a.iter()
            .zip(b)
            .map(|(x, y)| x.total_cmp(y))
            .find(|o| o.is_ne())
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    sols.dedup_by(|a, b| {
        a.iter()
            .zip(b.iter())
            .all(|(x, y)| (x - y).abs() <= 1e-9 * x.abs().max(y.abs()).max(1e-300))
    });
    sols
}

fn eliminate(
    eqs: &[Expr],
    xs: &[String],
    positive: &dyn Fn(&str) -> bool,
    depth: usize,
) -> Vec<Vec<f64>> {
    if xs.len() == 1 {
        let x = &xs[0];
        let mut roots: Vec<f64> = Vec::new();
        for (i, e) in eqs.iter().enumerate() {
            let rs = solve1(e, x, positive(x));
            let ok: Vec<f64> = rs
                .into_iter()
                .filter(|&r| {
                    eqs.iter()
                        .enumerate()
                        .all(|(j, o)| j == i || residual_ok(o, x, r))
                })
                .collect();
            if !ok.is_empty() {
                roots = ok;
                break;
            }
        }
        return roots.into_iter().map(|r| vec![r]).collect();
    }
    if depth > 6 {
        return vec![];
    }
    for (i, e) in eqs.iter().enumerate() {
        for (k, x) in xs.iter().enumerate() {
            if e.count(x) != 1 {
                continue;
            }
            let mut out = Vec::new();
            for sol_x in isolate(e, x, n(0.0)) {
                let rest_eqs: Vec<Expr> = eqs
                    .iter()
                    .enumerate()
                    .filter(|(j, _)| *j != i)
                    .map(|(_, o)| subst_var(o, x, &sol_x))
                    .collect();
                let rest_xs: Vec<String> = xs.iter().filter(|v| *v != x).cloned().collect();
                for sub in eliminate(&rest_eqs, &rest_xs, positive, depth + 1) {
                    let vals: HashMap<String, f64> =
                        rest_xs.iter().cloned().zip(sub.iter().copied()).collect();
                    let Some(xv) = sol_x.eval_map(&vals) else {
                        continue;
                    };
                    if !xv.is_finite() || (positive(x) && xv < 0.0) {
                        continue;
                    }
                    let mut full = sub.clone();
                    full.insert(k, xv);
                    out.push(full);
                }
            }
            if !out.is_empty() {
                return out
                    .into_iter()
                    .filter(|t| {
                        let vals: HashMap<String, f64> =
                            xs.iter().cloned().zip(t.iter().copied()).collect();
                        eqs.iter().all(|e| {
                            let f = e.eval_map(&vals).unwrap_or(f64::NAN);
                            let scale = term_scale_map(e, &vals).max(1e-300);
                            f.is_finite() && f.abs() <= 1e-7 * scale
                        })
                    })
                    .collect();
            }
        }
    }
    vec![]
}

fn term_scale_map(e: &Expr, vals: &HashMap<String, f64>) -> f64 {
    abs_eval(e, &|v| vals.get(v).copied())
}

fn newton(eqs: &[Expr], xs: &[String], positive: &dyn Fn(&str) -> bool) -> Vec<Vec<f64>> {
    let nx = xs.len();
    let eval = |v: &[f64]| -> Option<Vec<f64>> {
        let m: HashMap<String, f64> = xs.iter().cloned().zip(v.iter().copied()).collect();
        eqs.iter().map(|e| e.eval_map(&m)).collect()
    };
    for start in [1.0, 10.0, 0.1, 100.0, 1000.0] {
        let mut v = vec![start; nx];
        for _ in 0..100 {
            let Some(f) = eval(&v) else { break };
            let mut jac = vec![vec![0.0; nx]; eqs.len()];
            for j in 0..nx {
                let h = 1e-7 * v[j].abs().max(1e-9);
                let mut vh = v.clone();
                vh[j] += h;
                let Some(fh) = eval(&vh) else { break };
                for i in 0..eqs.len() {
                    jac[i][j] = (fh[i] - f[i]) / h;
                }
            }
            let Some(step) = lu_solve(jac, f.clone()) else {
                break;
            };
            for j in 0..nx {
                v[j] -= step[j];
            }
            if step
                .iter()
                .zip(&v)
                .all(|(s, x)| s.abs() <= 1e-13 * x.abs().max(1e-300))
            {
                break;
            }
        }
        let m: HashMap<String, f64> = xs.iter().cloned().zip(v.iter().copied()).collect();
        let ok = eqs.iter().all(|e| {
            let f = e.eval_map(&m).unwrap_or(f64::NAN);
            f.is_finite() && f.abs() <= 1e-7 * term_scale_map(e, &m).max(1e-300)
        }) && xs
            .iter()
            .zip(&v)
            .all(|(x, val)| !positive(x) || *val >= 0.0);
        if ok {
            return vec![v];
        }
    }
    vec![]
}

fn lu_solve(mut a: Vec<Vec<f64>>, mut b: Vec<f64>) -> Option<Vec<f64>> {
    let n = b.len();
    if a.len() != n {
        return None;
    }
    for c in 0..n {
        let p = (c..n).max_by(|&i, &j| a[i][c].abs().total_cmp(&a[j][c].abs()))?;
        if a[p][c] == 0.0 {
            return None;
        }
        a.swap(c, p);
        b.swap(c, p);
        for r in c + 1..n {
            let (top, bottom) = a.split_at_mut(r);
            let (pivot, row) = (&top[c], &mut bottom[0]);
            let k = row[c] / pivot[c];
            for (x, p) in row[c..].iter_mut().zip(&pivot[c..]) {
                *x -= k * p;
            }
            b[r] -= k * b[c];
        }
    }
    let mut x = vec![0.0; n];
    for r in (0..n).rev() {
        let s: f64 = (r + 1..n).map(|j| a[r][j] * x[j]).sum();
        x[r] = (b[r] - s) / a[r][r];
    }
    Some(x)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::expr::{parse, parse_equation};

    fn s1(src: &str, x: &str, positive: bool) -> Vec<f64> {
        solve1(&parse_equation(src).unwrap(), x, positive)
    }

    fn close(a: &[f64], b: &[f64]) -> bool {
        a.len() == b.len()
            && a.iter()
                .zip(b)
                .all(|(x, y)| (x - y).abs() <= 1e-6 * y.abs().max(1e-12))
    }

    #[test]
    fn inversion_covers_transcendental_formulas() {
        // ts_discharge for R: 60 = 396 exp(-5/(R*1.8e-3))
        let r = s1("60 = 396*exp(-5/(R*1.8e-3))", "R", true);
        assert!(close(&r, &[1472.0059639569]), "{r:?}");
        assert!(close(
            &s1("phi = -atan(200/300)", "phi", false),
            &[-0.588002603547568]
        ));
        assert!(close(
            &s1("0.5 = cos(x)", "x", false),
            &[1.0471975511965976, 5.235987755982989]
        ));
        assert!(close(&s1("9 = x**2", "x", false), &[-3.0, 3.0]));
        assert!(close(&s1("9 = x**2", "x", true), &[3.0]));
        assert!(close(&s1("-8 = x**3", "x", false), &[-2.0]));
    }

    #[test]
    fn polynomials_keep_both_roots_and_double_roots() {
        // battery_load reduced: P = I*(V_oc - I*R)
        let r = s1("30000 = I*(391.4 - I*0.08)", "I", true);
        assert!(close(&r, &[77.8878945567, 4814.6121054433]), "{r:?}");
        // touching root, no sign change
        assert!(close(&s1("0 = (x - 2)**2", "x", true), &[2.0]));
        // degree 6 via Aberth
        let r = s1("0 = (x-1)*(x-2)*(x-3)*(x-4)*(x-5)*(x-6)", "x", true);
        assert!(close(&r, &[1.0, 2.0, 3.0, 4.0, 5.0, 6.0]), "{r:?}");
    }

    #[test]
    fn extreme_magnitudes_and_zero() {
        assert!(close(&s1("2.1e11 = F*1/(1e-9)", "F", true), &[210.0]));
        assert!(close(
            &s1("40.84 = 1/(2*pi*433e6*C)", "C", true),
            &[9.000082e-12]
        ));
        assert_eq!(
            s1("frac = 1/2 - ay*0.25/(9.81*1.2)", "ay", false).len(),
            0,
            "two unknowns -> not solvable here"
        );
        assert!(close(
            &s1("0.5 = 1/2 - ay*0.25/(9.81*1.2)", "ay", false),
            &[0.0]
        ));
    }

    #[test]
    fn scan_handles_mixed_and_rejects_poles() {
        // self-heating fixed point: T = 298.15 + 13**2 * (1*(1 + 1e-5*(T - 298.15))) * 1
        let r = s1("T = 298.15 + 169*(1 + 1e-5*(T - 298.15))", "T", true);
        assert!(close(&r, &[467.43614]), "{r:?}");
        // vswr: VSWR = (1+G)/(1-G) has a pole at G = 1; for VSWR = 3, G = 0.5 only
        assert!(close(&s1("3 = (1 + Gm)/(1 - Gm)", "Gm", true), &[0.5]));
        // x*exp(x) = 5 (not invertible, not polynomial)
        assert!(close(&s1("x*exp(x) = 5", "x", true), &[1.3267246652422002]));
    }

    #[test]
    fn systems_stay_paired() {
        // V_t = 391.4 - I*0.08, 30000 = I*V_t
        let eqs = [
            parse_equation("V_t = 391.4 - I*0.08").unwrap(),
            parse_equation("30000 = I*V_t").unwrap(),
        ];
        let sols = solve_system(&eqs, &["V_t".into(), "I".into()], &|_| true);
        assert_eq!(sols.len(), 2);
        for s in &sols {
            let (i, vt) = (s[0], s[1]); // sorted by name: I, V_t
            assert!((vt - (391.4 - i * 0.08)).abs() < 1e-6);
        }
        assert!((sols[0][0] - 77.88786).abs() < 1e-4);
        // suvat: v = a*t, 75 = a*t**2/2 with v = 27.7778
        let eqs = [
            parse_equation("27.7777777778 = a*t").unwrap(),
            parse_equation("75 = a*t**2/2").unwrap(),
        ];
        let sols = solve_system(&eqs, &["a".into(), "t".into()], &|_| true);
        assert!(close(&sols[0], &[5.1440329218, 5.4]), "{sols:?}");
        let _ = parse("x");
    }
}
