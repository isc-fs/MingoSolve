//! LaTeX for equations and variable names, so the app can typeset `P = I**2*R` as P = I²R: fractions, powers and
//! roots, Greek letters and subscripts (`rho_air` → ρ_air, `v0` → v₀, `dT` → ΔT), brackets only where needed.

use crate::expr::{parse, Expr, Func, Op};

const GREEK: &[(&str, &str)] = &[
    ("alpha", "\\alpha"),
    ("beta", "\\beta"),
    ("gamma", "\\gamma"),
    ("delta", "\\delta"),
    ("epsilon", "\\varepsilon"),
    ("eps", "\\varepsilon"),
    ("zeta", "\\zeta"),
    ("eta", "\\eta"),
    ("theta", "\\theta"),
    ("kappa", "\\kappa"),
    ("lambda", "\\lambda"),
    ("lam", "\\lambda"),
    ("mu", "\\mu"),
    ("nu", "\\nu"),
    ("xi", "\\xi"),
    ("rho", "\\rho"),
    ("sigma", "\\sigma"),
    ("tau", "\\tau"),
    ("phi", "\\varphi"),
    ("chi", "\\chi"),
    ("psi", "\\psi"),
    ("omega", "\\omega"),
    ("Gamma", "\\Gamma"),
    ("Delta", "\\Delta"),
    ("Theta", "\\Theta"),
    ("Lambda", "\\Lambda"),
    ("Sigma", "\\Sigma"),
    ("Phi", "\\Phi"),
    ("Psi", "\\Psi"),
    ("Omega", "\\Omega"),
];

fn greek(s: &str) -> Option<&'static str> {
    GREEK.iter().find(|(n, _)| *n == s).map(|(_, t)| *t)
}

/// A symbol without subscript: Greek letters, `Qdot` → Q̇, one letter as is, longer names upright (`COP`, `CdA`).
fn base(s: &str) -> String {
    if let Some(g) = greek(s) {
        return g.to_string();
    }
    if let Some(b) = s.strip_suffix("dot").filter(|b| !b.is_empty()) {
        return format!("\\dot{{{}}}", base(b));
    }
    if s.chars().count() == 1 {
        return s.to_string();
    }
    format!("\\mathrm{{{s}}}")
}

fn subscript(s: &str) -> String {
    if s.chars().all(|c| c.is_ascii_digit()) || s.chars().count() == 1 {
        return s.to_string();
    }
    if let Some(g) = greek(s) {
        return g.to_string();
    }
    format!("\\mathrm{{{}}}", s.replace('_', ","))
}

/// A variable name as LaTeX.
pub fn var_latex(name: &str) -> String {
    if name == "dIdt" {
        return "\\frac{dI}{dt}".into();
    }
    // a leading d before a capital (dT, dSoC, dFz_x) or the height/position names dh, dz is a difference
    if let Some(rest) = name.strip_prefix('d') {
        let difference = rest.starts_with(|c: char| c.is_ascii_uppercase())
            || ["h", "z"].contains(&rest.split('_').next().unwrap_or(""));
        if difference {
            return format!("\\Delta {}", var_latex(rest));
        }
    }
    if let Some((b, sub)) = name.split_once('_') {
        return format!("{}_{{{}}}", base(b), subscript(sub));
    }
    // trailing digits are a subscript: v0, C1, R25
    let digits = name.len() - name.trim_end_matches(|c: char| c.is_ascii_digit()).len();
    if digits > 0 && digits < name.len() {
        let (b, n) = name.split_at(name.len() - digits);
        return format!("{}_{{{n}}}", base(b));
    }
    base(name)
}

fn number(n: f64) -> String {
    if n == std::f64::consts::PI {
        return "\\pi".into();
    }
    let a = n.abs();
    if a != 0.0 && !(1e-3..1e5).contains(&a) {
        let exp = a.log10().floor() as i32;
        let mantissa = n / 10f64.powi(exp);
        let m = format!("{}", (mantissa * 1e9).round() / 1e9);
        return if m == "1" {
            format!("10^{{{exp}}}")
        } else {
            format!("{m} \\times 10^{{{exp}}}")
        };
    }
    format!("{n}")
}

fn is_sum(e: &Expr) -> bool {
    matches!(e, Expr::Bin(Op::Add | Op::Sub, ..))
}

fn paren(s: String) -> String {
    format!("\\left({s}\\right)")
}

fn factors<'a>(e: &'a Expr, out: &mut Vec<&'a Expr>) {
    match e {
        Expr::Bin(Op::Mul, a, b) => {
            factors(a, out);
            factors(b, out);
        }
        _ => out.push(e),
    }
}

fn is_half(e: &Expr) -> bool {
    match e {
        Expr::Num(n) => *n == 0.5,
        Expr::Bin(Op::Div, a, b) => {
            matches!((&**a, &**b), (Expr::Num(x), Expr::Num(y)) if *x == 1.0 && *y == 2.0)
        }
        _ => false,
    }
}

/// An expression as LaTeX.
pub fn expr_latex(e: &Expr) -> String {
    match e {
        Expr::Num(n) => number(*n),
        Expr::Var(v) => var_latex(v),
        Expr::Neg(x) => {
            let inner = expr_latex(x);
            format!("-{}", if is_sum(x) { paren(inner) } else { inner })
        }
        Expr::Call(f, x) => {
            let arg = expr_latex(x);
            match f {
                Func::Sqrt => format!("\\sqrt{{{arg}}}"),
                Func::Abs => format!("\\left|{arg}\\right|"),
                // fractions in a superscript get too small to read: write exp(...) instead
                Func::Exp if !has_fraction(x) => format!("e^{{{arg}}}"),
                Func::Exp => format!("\\exp{}", paren(arg)),
                Func::Ln => format!("\\ln{}", paren(arg)),
                Func::Sin => format!("\\sin{}", paren(arg)),
                Func::Cos => format!("\\cos{}", paren(arg)),
                Func::Tan => format!("\\tan{}", paren(arg)),
                Func::Asin => format!("\\arcsin{}", paren(arg)),
                Func::Acos => format!("\\arccos{}", paren(arg)),
                Func::Atan => format!("\\arctan{}", paren(arg)),
            }
        }
        Expr::Bin(op, a, b) => match op {
            Op::Add => match &**b {
                Expr::Neg(nb) => format!("{} - {}", expr_latex(a), operand_of_minus(nb)),
                _ => format!("{} + {}", expr_latex(a), expr_latex(b)),
            },
            Op::Sub => format!("{} - {}", expr_latex(a), operand_of_minus(b)),
            Op::Div => format!("\\frac{{{}}}{{{}}}", expr_latex(a), expr_latex(b)),
            Op::Pow => {
                if is_half(b) {
                    return format!("\\sqrt{{{}}}", expr_latex(a));
                }
                let bracket = matches!(**a, Expr::Bin(..) | Expr::Neg(_) | Expr::Call(..))
                    || matches!(**a, Expr::Num(n) if n < 0.0);
                let base = if bracket {
                    paren(expr_latex(a))
                } else {
                    expr_latex(a)
                };
                format!("{base}^{{{}}}", expr_latex(b))
            }
            Op::Mul => {
                let mut fs = Vec::new();
                factors(e, &mut fs);
                let mut out = String::new();
                for (i, f) in fs.iter().enumerate() {
                    let s = expr_latex(f);
                    let s = if is_sum(f) || (i > 0 && matches!(f, Expr::Neg(_))) {
                        paren(s)
                    } else {
                        s
                    };
                    if i > 0 {
                        // numbers after the first factor need a visible product sign; letters just sit together
                        let number_next = matches!(f, Expr::Num(n) if *n != std::f64::consts::PI);
                        out.push_str(if number_next { " \\cdot " } else { "\\," });
                    }
                    out.push_str(&s);
                }
                out
            }
        },
    }
}

fn has_fraction(e: &Expr) -> bool {
    match e {
        Expr::Bin(Op::Div, ..) => true,
        Expr::Bin(_, a, b) => has_fraction(a) || has_fraction(b),
        Expr::Neg(x) | Expr::Call(_, x) => has_fraction(x),
        _ => false,
    }
}

fn operand_of_minus(e: &Expr) -> String {
    let s = expr_latex(e);
    if is_sum(e) || matches!(e, Expr::Neg(_)) {
        paren(s)
    } else {
        s
    }
}

/// An equation source (`lhs = rhs`) as LaTeX; the source itself if it doesn't parse.
pub fn equation_latex(src: &str) -> String {
    let Some((l, r)) = src.split_once('=') else {
        return src.to_string();
    };
    match (parse(l), parse(r)) {
        (Ok(l), Ok(r)) => format!("{} = {}", expr_latex(&l), expr_latex(&r)),
        _ => src.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn powers_products_and_fractions_read_like_a_textbook() {
        assert_eq!(equation_latex("P = I**2*R"), "P = I^{2}\\,R");
        assert_eq!(
            equation_latex("F_L = 0.5*rho*ClA*v**2"),
            "F_{L} = 0.5\\,\\rho\\,\\mathrm{ClA}\\,v^{2}"
        );
        assert_eq!(
            equation_latex("v_avg = s/t"),
            "v_{\\mathrm{avg}} = \\frac{s}{t}"
        );
        assert_eq!(equation_latex("x = 2*pi*r"), "x = 2\\,\\pi\\,r");
        assert_eq!(equation_latex("y = a*2"), "y = a \\cdot 2");
    }

    #[test]
    fn brackets_appear_exactly_where_precedence_needs_them() {
        assert_eq!(
            expr_latex(&parse("a*(b + c)").unwrap()),
            "a\\,\\left(b + c\\right)"
        );
        assert_eq!(
            expr_latex(&parse("(a + b)**2").unwrap()),
            "\\left(a + b\\right)^{2}"
        );
        assert_eq!(
            expr_latex(&parse("a - (b - c)").unwrap()),
            "a - \\left(b - c\\right)"
        );
        assert_eq!(expr_latex(&parse("a - b - c").unwrap()), "a - b - c");
        assert_eq!(
            expr_latex(&parse("(a + b)/(c + d)").unwrap()),
            "\\frac{a + b}{c + d}"
        );
        // -x**2 is -(x²), (-x)**2 is (−x)²: different numbers, different typesetting
        assert_eq!(expr_latex(&parse("-x**2").unwrap()), "-x^{2}");
        assert_eq!(
            expr_latex(&parse("(-x)**2").unwrap()),
            "\\left(-x\\right)^{2}"
        );
        assert_eq!(expr_latex(&parse("a**(b + c)").unwrap()), "a^{b + c}");
    }

    #[test]
    fn roots_exponentials_and_functions() {
        assert_eq!(expr_latex(&parse("(g*R)**0.5").unwrap()), "\\sqrt{g\\,R}");
        assert_eq!(
            expr_latex(&parse("sqrt(2*a*s)").unwrap()),
            "\\sqrt{2\\,a\\,s}"
        );
        assert_eq!(
            equation_latex("R_T = R_25*exp(B_ntc*(1/T - 1/T_25))"),
            "R_{T} = R_{25}\\,\\exp\\left(B_{\\mathrm{ntc}}\\,\\left(\\frac{1}{T} - \\frac{1}{T_{25}}\\right)\\right)"
        );
        assert_eq!(
            expr_latex(&parse("atan(L/R)").unwrap()),
            "\\arctan\\left(\\frac{L}{R}\\right)"
        );
    }

    #[test]
    fn names_become_symbols() {
        assert_eq!(var_latex("rho_air"), "\\rho_{\\mathrm{air}}");
        assert_eq!(var_latex("eta_g"), "\\eta_{g}");
        assert_eq!(var_latex("v0"), "v_{0}");
        assert_eq!(var_latex("R25"), "R_{25}");
        assert_eq!(var_latex("dT"), "\\Delta T");
        assert_eq!(var_latex("dSoC"), "\\Delta \\mathrm{SoC}");
        assert_eq!(var_latex("dh"), "\\Delta h");
        assert_eq!(var_latex("d"), "d");
        assert_eq!(var_latex("Qdot_h"), "\\dot{Q}_{h}");
        assert_eq!(var_latex("COP"), "\\mathrm{COP}");
        assert_eq!(var_latex("mu"), "\\mu");
        assert_eq!(
            number(1.2566370614359173e-06),
            "1.256637061 \\times 10^{-6}"
        );
        assert_eq!(number(1e-7), "10^{-7}");
    }

    #[test]
    fn every_shipped_equation_renders() {
        for f in &crate::registry::registry().formulas {
            for src in &f.src {
                let tex = equation_latex(src);
                assert_ne!(&tex, src, "{}: {src} did not convert", f.key);
                assert!(
                    !tex.contains("**") && !tex.contains('*'),
                    "{}: leftover operator in {tex}",
                    f.key
                );
            }
        }
    }
}
