//! Number formatting shared by the CLI and the app: `g6` mimics C/Python `%.6g` (the Python engine's output),
//! so values print the same in both engines.

pub fn g(x: f64, sig: usize) -> String {
    if x == 0.0 {
        return "0".into();
    }
    if !x.is_finite() {
        return format!("{x}");
    }
    let exp = x.abs().log10().floor() as i32;
    let p = sig.max(1) as i32;
    if exp < -4 || exp >= p {
        let s = format!("{:.*e}", (p - 1) as usize, x);
        let (mant, e) = s.split_once('e').unwrap();
        let mant = trim_zeros(mant);
        let e: i32 = e.parse().unwrap();
        format!("{mant}e{}{:02}", if e < 0 { '-' } else { '+' }, e.abs())
    } else {
        trim_zeros(&format!("{:.*}", (p - 1 - exp).max(0) as usize, x))
    }
}

pub fn g6(x: f64) -> String {
    g(x, 6)
}

fn trim_zeros(s: &str) -> String {
    if s.contains('.') {
        s.trim_end_matches('0').trim_end_matches('.').to_string()
    } else {
        s.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_percent_g() {
        assert_eq!(g6(77.88789455666219), "77.8879");
        assert_eq!(g6(0.01736111), "0.0173611");
        assert_eq!(g6(115741.0), "115741");
        assert_eq!(g6(1234567.0), "1.23457e+06");
        assert_eq!(g6(0.00001234), "1.234e-05");
        assert_eq!(g6(-33.69006752597979), "-33.6901");
        assert_eq!(g6(5.4), "5.4");
        assert_eq!(g6(19.0), "19");
        assert_eq!(g6(0.0), "0");
    }
}
