//! Expression trees: tokenizer + precedence-climbing parser, evaluation, substitution and inspection helpers.
//! Shared by equations (`v = v0 + a*t`) and unit quantities (`10mm**2`, `4.2V*20Ah`), where a number directly
//! followed by a name or `(` is an implicit product. `**` is power; `^` is rejected (sympy reads it as XOR).

use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Func {
    Exp,
    Ln,
    Sqrt,
    Sin,
    Cos,
    Tan,
    Asin,
    Acos,
    Atan,
    Abs,
}

impl Func {
    fn from_name(name: &str) -> Option<Self> {
        Some(match name {
            "exp" => Self::Exp,
            "log" | "ln" => Self::Ln,
            "sqrt" => Self::Sqrt,
            "sin" => Self::Sin,
            "cos" => Self::Cos,
            "tan" => Self::Tan,
            "asin" => Self::Asin,
            "acos" => Self::Acos,
            "atan" => Self::Atan,
            "abs" => Self::Abs,
            _ => return None,
        })
    }

    pub fn apply(self, x: f64) -> f64 {
        match self {
            Self::Exp => x.exp(),
            Self::Ln => x.ln(),
            Self::Sqrt => x.sqrt(),
            Self::Sin => x.sin(),
            Self::Cos => x.cos(),
            Self::Tan => x.tan(),
            Self::Asin => x.asin(),
            Self::Acos => x.acos(),
            Self::Atan => x.atan(),
            Self::Abs => x.abs(),
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Exp => "exp",
            Self::Ln => "log",
            Self::Sqrt => "sqrt",
            Self::Sin => "sin",
            Self::Cos => "cos",
            Self::Tan => "tan",
            Self::Asin => "asin",
            Self::Acos => "acos",
            Self::Atan => "atan",
            Self::Abs => "abs",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Op {
    Add,
    Sub,
    Mul,
    Div,
    Pow,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Num(f64),
    Var(String),
    Neg(Box<Expr>),
    Bin(Op, Box<Expr>, Box<Expr>),
    Call(Func, Box<Expr>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParseError(pub String);

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for ParseError {}

#[derive(Debug, Clone, PartialEq)]
enum Tok {
    Num(f64),
    Ident(String),
    Op(char),
    Pow,
    LParen,
    RParen,
}

impl fmt::Display for Tok {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Num(n) => write!(f, "number {n}"),
            Self::Ident(s) => f.write_str(s),
            Self::Op(c) => write!(f, "{c}"),
            Self::Pow => f.write_str("**"),
            Self::LParen => f.write_str("("),
            Self::RParen => f.write_str(")"),
        }
    }
}

fn ident_char(c: char, first: bool) -> bool {
    c.is_alphabetic()
        || c == '_'
        || c == '%'
        || c == 'µ'
        || c == 'Ω'
        || (!first && c.is_ascii_digit())
}

fn tokenize(src: &str) -> Result<Vec<Tok>, ParseError> {
    let chars: Vec<char> = src.chars().collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if c.is_whitespace() {
            i += 1;
        } else if c.is_ascii_digit()
            || (c == '.' && chars.get(i + 1).is_some_and(|d| d.is_ascii_digit()))
        {
            let start = i;
            while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == '.') {
                i += 1;
            }
            if i < chars.len() && (chars[i] == 'e' || chars[i] == 'E') {
                let mut j = i + 1;
                if j < chars.len() && (chars[j] == '+' || chars[j] == '-') {
                    j += 1;
                }
                if j < chars.len() && chars[j].is_ascii_digit() {
                    i = j;
                    while i < chars.len() && chars[i].is_ascii_digit() {
                        i += 1;
                    }
                }
            }
            let text: String = chars[start..i].iter().collect();
            let n: f64 = text
                .parse()
                .map_err(|_| ParseError(format!("bad number {text:?}")))?;
            if !n.is_finite() {
                return Err(ParseError(format!("number {text} is too large")));
            }
            out.push(Tok::Num(n));
        } else if ident_char(c, true) {
            let start = i;
            while i < chars.len() && ident_char(chars[i], false) {
                i += 1;
            }
            out.push(Tok::Ident(chars[start..i].iter().collect()));
        } else if c == '*' && chars.get(i + 1) == Some(&'*') {
            out.push(Tok::Pow);
            i += 2;
        } else if "+-*/".contains(c) {
            out.push(Tok::Op(c));
            i += 1;
        } else if c == '(' {
            out.push(Tok::LParen);
            i += 1;
        } else if c == ')' {
            out.push(Tok::RParen);
            i += 1;
        } else if c == '^' {
            return Err(ParseError("use ** for powers (^ is not allowed)".into()));
        } else {
            return Err(ParseError(format!("unexpected character {c:?}")));
        }
    }
    // implicit product: number followed by a name or '(' (10mm, 1.5g0, 2(x+1))
    let mut with_implicit = Vec::with_capacity(out.len());
    for (k, t) in out.iter().enumerate() {
        with_implicit.push(t.clone());
        if matches!(t, Tok::Num(_)) && matches!(out.get(k + 1), Some(Tok::Ident(_) | Tok::LParen)) {
            with_implicit.push(Tok::Op('*'));
        }
    }
    Ok(with_implicit)
}

/// Deepest tree (and deepest nesting of brackets or unary signs) the parser accepts. Evaluation, display and drop
/// recurse over the tree, so this keeps every user-reachable walk far inside a 2 MiB thread stack.
pub const MAX_DEPTH: usize = 256;

const TOO_DEEP: &str = "expression nested too deeply";

/// A subtree and its height, so depth is known without walking it again.
type Node = (Expr, usize);

struct Parser {
    toks: Vec<Tok>,
    pos: usize,
    nest: usize,
}

fn node(e: Expr, height: usize) -> Result<Node, ParseError> {
    if height > MAX_DEPTH {
        return Err(ParseError(TOO_DEEP.into()));
    }
    Ok((e, height))
}

fn bin(op: Op, a: Node, b: Node) -> Result<Node, ParseError> {
    let h = a.1.max(b.1) + 1;
    node(Expr::Bin(op, Box::new(a.0), Box::new(b.0)), h)
}

impl Parser {
    fn peek(&self) -> Option<&Tok> {
        self.toks.get(self.pos)
    }

    fn next(&mut self) -> Option<Tok> {
        let t = self.toks.get(self.pos).cloned();
        self.pos += 1;
        t
    }

    fn sum(&mut self) -> Result<Node, ParseError> {
        let mut lhs = self.product()?;
        while let Some(Tok::Op(c @ ('+' | '-'))) = self.peek().cloned() {
            self.pos += 1;
            let rhs = self.product()?;
            lhs = bin(if c == '+' { Op::Add } else { Op::Sub }, lhs, rhs)?;
        }
        Ok(lhs)
    }

    fn product(&mut self) -> Result<Node, ParseError> {
        let mut lhs = self.unary()?;
        while let Some(Tok::Op(c @ ('*' | '/'))) = self.peek().cloned() {
            self.pos += 1;
            let rhs = self.unary()?;
            lhs = bin(if c == '*' { Op::Mul } else { Op::Div }, lhs, rhs)?;
        }
        Ok(lhs)
    }

    /// Every recursive cycle of the grammar (brackets, calls, signs, `**` chains) passes through here.
    fn unary(&mut self) -> Result<Node, ParseError> {
        self.nest += 1;
        if self.nest > MAX_DEPTH {
            return Err(ParseError(TOO_DEEP.into()));
        }
        let r = match self.peek() {
            Some(Tok::Op('-')) => {
                self.pos += 1;
                self.unary()
                    .and_then(|(e, h)| node(Expr::Neg(Box::new(e)), h + 1))
            }
            Some(Tok::Op('+')) => {
                self.pos += 1;
                self.unary()
            }
            _ => self.power(),
        };
        self.nest -= 1;
        r
    }

    fn power(&mut self) -> Result<Node, ParseError> {
        let base = self.atom()?;
        if self.peek() == Some(&Tok::Pow) {
            self.pos += 1;
            let exp = self.unary()?; // right associative, and 2**-1 works
            return bin(Op::Pow, base, exp);
        }
        Ok(base)
    }

    fn atom(&mut self) -> Result<Node, ParseError> {
        match self.next() {
            Some(Tok::Num(n)) => Ok((Expr::Num(n), 0)),
            Some(Tok::Ident(name)) => {
                if self.peek() == Some(&Tok::LParen) {
                    let f = Func::from_name(&name)
                        .ok_or_else(|| ParseError(format!("unknown function {name}()")))?;
                    self.pos += 1;
                    let (arg, h) = self.sum()?;
                    self.expect_rparen()?;
                    node(Expr::Call(f, Box::new(arg)), h + 1)
                } else if name == "pi" {
                    Ok((Expr::Num(std::f64::consts::PI), 0))
                } else {
                    Ok((Expr::Var(name), 0))
                }
            }
            Some(Tok::LParen) => {
                let e = self.sum()?;
                self.expect_rparen()?;
                Ok(e)
            }
            Some(t) => Err(ParseError(format!("unexpected {t}"))),
            None => Err(ParseError("unexpected end of expression".into())),
        }
    }

    fn expect_rparen(&mut self) -> Result<(), ParseError> {
        match self.next() {
            Some(Tok::RParen) => Ok(()),
            _ => Err(ParseError("missing )".into())),
        }
    }
}

pub fn parse(src: &str) -> Result<Expr, ParseError> {
    let mut p = Parser {
        toks: tokenize(src)?,
        pos: 0,
        nest: 0,
    };
    let (e, _) = p.sum()?;
    if p.pos < p.toks.len() {
        return Err(ParseError(format!("unexpected {}", p.toks[p.pos])));
    }
    Ok(e)
}

/// `lhs = rhs` parsed as the residual `lhs - rhs`.
pub fn parse_equation(src: &str) -> Result<Expr, ParseError> {
    let (lhs, rhs) = src
        .split_once('=')
        .ok_or_else(|| ParseError(format!("not an equation: {src}")))?;
    Ok(Expr::Bin(
        Op::Sub,
        Box::new(parse(lhs)?),
        Box::new(parse(rhs)?),
    ))
}

impl Expr {
    pub fn eval(&self, env: &dyn Fn(&str) -> Option<f64>) -> Option<f64> {
        Some(match self {
            Self::Num(n) => *n,
            Self::Var(v) => env(v)?,
            Self::Neg(e) => -e.eval(env)?,
            Self::Call(f, e) => f.apply(e.eval(env)?),
            Self::Bin(op, a, b) => {
                let (a, b) = (a.eval(env)?, b.eval(env)?);
                match op {
                    Op::Add => a + b,
                    Op::Sub => a - b,
                    Op::Mul => a * b,
                    Op::Div => a / b,
                    Op::Pow => a.powf(b),
                }
            }
        })
    }

    pub fn eval_map(&self, vals: &HashMap<String, f64>) -> Option<f64> {
        self.eval(&|n| vals.get(n).copied())
    }

    pub fn vars(&self, out: &mut Vec<String>) {
        match self {
            Self::Num(_) => {}
            Self::Var(v) => {
                if !out.contains(v) {
                    out.push(v.clone());
                }
            }
            Self::Neg(e) | Self::Call(_, e) => e.vars(out),
            Self::Bin(_, a, b) => {
                a.vars(out);
                b.vars(out);
            }
        }
    }

    pub fn count(&self, var: &str) -> usize {
        match self {
            Self::Num(_) => 0,
            Self::Var(v) => usize::from(v == var),
            Self::Neg(e) | Self::Call(_, e) => e.count(var),
            Self::Bin(_, a, b) => a.count(var) + b.count(var),
        }
    }

    /// Replace known variables by numbers and fold constant subtrees.
    pub fn substitute(&self, vals: &HashMap<String, f64>) -> Expr {
        match self {
            Self::Num(n) => Self::Num(*n),
            Self::Var(v) => vals.get(v).map_or_else(|| self.clone(), |x| Self::Num(*x)),
            Self::Neg(e) => match e.substitute(vals) {
                Self::Num(n) => Self::Num(-n),
                e => Self::Neg(Box::new(e)),
            },
            Self::Call(f, e) => match e.substitute(vals) {
                Self::Num(n) => Self::Num(f.apply(n)),
                e => Self::Call(*f, Box::new(e)),
            },
            Self::Bin(op, a, b) => {
                let (a, b) = (a.substitute(vals), b.substitute(vals));
                if let (Self::Num(_), Self::Num(_)) = (&a, &b) {
                    let folded = Self::Bin(*op, Box::new(a), Box::new(b));
                    return Self::Num(folded.eval(&|_| None).expect("constant"));
                }
                Self::Bin(*op, Box::new(a), Box::new(b))
            }
        }
    }
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Num(n) => write!(f, "{n}"),
            Self::Var(v) => f.write_str(v),
            Self::Neg(e) => write!(f, "-({e})"),
            Self::Call(func, e) => write!(f, "{}({e})", func.name()),
            Self::Bin(op, a, b) => {
                let s = match op {
                    Op::Add => "+",
                    Op::Sub => "-",
                    Op::Mul => "*",
                    Op::Div => "/",
                    Op::Pow => "**",
                };
                write!(f, "({a} {s} {b})")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ev(src: &str) -> f64 {
        parse(src).unwrap().eval(&|_| None).unwrap()
    }

    fn ev_with(src: &str, vals: &[(&str, f64)]) -> f64 {
        let m: HashMap<String, f64> = vals.iter().map(|(k, v)| (k.to_string(), *v)).collect();
        parse(src).unwrap().eval_map(&m).unwrap()
    }

    #[test]
    fn precedence_and_associativity() {
        assert_eq!(ev("1 + 2*3"), 7.0);
        assert_eq!(ev("2**3**2"), 512.0);
        assert_eq!(ev("-2**2"), -4.0);
        assert_eq!(ev("2**-1"), 0.5);
        assert_eq!(ev("8/4/2"), 1.0);
        assert_eq!(ev("10 - 4 - 3"), 3.0);
        assert!((ev("2*pi") - std::f64::consts::TAU).abs() < 1e-12);
    }

    #[test]
    fn numbers_exponents_and_implicit_products() {
        assert_eq!(ev("1e3"), 1000.0);
        assert_eq!(ev("2.5E-2"), 0.025);
        assert_eq!(ev(".5"), 0.5);
        assert_eq!(ev("2(3+1)"), 8.0);
        assert_eq!(ev_with("10mm**2", &[("mm", 0.001)]), 10.0 * 1e-6);
        assert_eq!(
            ev_with("4.2V*20Ah", &[("V", 1.0), ("Ah", 3600.0)]),
            4.2 * 20.0 * 3600.0
        );
        assert_eq!(ev_with("1.5g0", &[("g0", 9.81)]), 1.5 * 9.81);
        assert_eq!(ev_with("3em", &[("em", 2.0)]), 6.0);
    }

    #[test]
    fn functions_and_equations() {
        assert!((ev("exp(log(5))") - 5.0).abs() < 1e-12);
        let r = parse_equation("v = v0 + a*t").unwrap();
        assert_eq!(
            ev_with_expr(&r, &[("v", 10.0), ("v0", 2.0), ("a", 4.0), ("t", 2.0)]),
            0.0
        );
        let mut vars = Vec::new();
        r.vars(&mut vars);
        assert_eq!(vars, ["v", "v0", "a", "t"]);
        assert_eq!(parse("V_c - V_0*exp(-t/(R*C))").unwrap().count("t"), 1);
    }

    fn ev_with_expr(e: &Expr, vals: &[(&str, f64)]) -> f64 {
        let m: HashMap<String, f64> = vals.iter().map(|(k, v)| (k.to_string(), *v)).collect();
        e.eval_map(&m).unwrap()
    }

    #[test]
    fn substitute_folds_constants() {
        let e = parse("a*t + v0").unwrap();
        let m: HashMap<String, f64> = [("a".to_string(), 2.0), ("v0".to_string(), 1.0)].into();
        assert_eq!(e.substitute(&m).to_string(), "((2 * t) + 1)");
        let all: HashMap<String, f64> = [("a", 2.0), ("v0", 1.0), ("t", 3.0)]
            .map(|(k, v)| (k.to_string(), v))
            .into();
        assert_eq!(e.substitute(&all), Expr::Num(7.0));
    }

    #[test]
    fn errors() {
        assert!(parse("2^3").is_err());
        assert!(parse("(1+2").is_err());
        assert!(parse("foo(1)").is_err());
        assert!(parse("1 +").is_err());
        assert!(parse_equation("a + b").is_err());
    }
}
