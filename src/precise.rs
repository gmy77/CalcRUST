//! Exact rational arithmetic for finite decimal operands and basic operations.
//! Transcendental/non-rational operations explicitly fall back to the f64 engine.
use crate::engine::{self, Angle, CalcResult, Expr};
use num_bigint::BigInt;
use num_rational::BigRational as R;
use num_traits::{One, Signed, ToPrimitive, Zero};
use std::{collections::BTreeMap, str::FromStr};

fn number(s: &str) -> Option<R> {
    if s.starts_with("0x") {
        return BigInt::parse_bytes(s[2..].as_bytes(), 16).map(R::from_integer);
    }
    let (mantissa, exponent) = if let Some((a, b)) = s.split_once(['e', 'E']) {
        (a, b.parse::<i32>().ok()?)
    } else {
        (s, 0)
    };
    if exponent.unsigned_abs() > 2048 || mantissa.len() > 4096 {
        return None;
    }
    let decimals = mantissa
        .split_once('.')
        .map(|(_, b)| b.len() as i32)
        .unwrap_or(0);
    let digits = mantissa.replace('.', "");
    let numerator = BigInt::from_str(&digits).ok()?;
    let scale = exponent - decimals;
    let ten = BigInt::from(10);
    Some(if scale >= 0 {
        R::from_integer(numerator * ten.pow(scale as u32))
    } else {
        R::new(numerator, ten.pow((-scale) as u32))
    })
}
fn exact(ast: &Expr) -> Option<CalcResult<R>> {
    macro_rules! get {
        ($e:expr) => {
            match exact($e) {
                Some(Ok(v)) => v,
                Some(Err(e)) => return Some(Err(e)),
                None => return None,
            }
        };
    }
    let value = match ast {
        Expr::Number(s) => match number(s) {
            Some(v) => v,
            None => return Some(Err("Numero non valido o esponente fuori intervallo".into())),
        },
        Expr::Variable(_) => return None,
        Expr::Unary(o, a) => {
            let a = get!(a);
            match o.as_str() {
                "-" => -a,
                "+" => a,
                _ => return None,
            }
        }
        Expr::Binary(o, a, b) => {
            let a = get!(a);
            let b = get!(b);
            match o.as_str() {
                "+" => a + b,
                "-" => a - b,
                "*" => a * b,
                "/" => {
                    if b.is_zero() {
                        return Some(Err("Impossibile dividere per zero".into()));
                    }
                    a / b
                }
                "mod" | "%" => {
                    if b.is_zero() {
                        return Some(Err("Impossibile dividere per zero".into()));
                    }
                    a % b
                }
                "^" => {
                    if !b.is_integer() {
                        return None;
                    }
                    let power = b.to_integer().to_i32()?;
                    if power.unsigned_abs() > 2048 {
                        return Some(Err("Esponente troppo grande per il calcolo esatto".into()));
                    }
                    if a.is_zero() && power <= 0 {
                        return Some(Err("Potenza non definita".into()));
                    }
                    a.pow(power)
                }
                _ => return None,
            }
        }
        Expr::Call(n, args) => {
            if args.len() != 1 {
                return None;
            }
            let a = get!(&args[0]);
            match n.as_str() {
                "sq" => &a * &a,
                "cube" => &a * &a * &a,
                "abs" => a.abs(),
                "inv" => {
                    if a.is_zero() {
                        return Some(Err("Impossibile dividere per zero".into()));
                    }
                    a.recip()
                }
                "floor" => a.floor(),
                "ceil" => a.ceil(),
                "round" => a.round(),
                "fact" => {
                    if !a.is_integer() {
                        return Some(Err("Fattoriale: serve un intero".into()));
                    }
                    let x = a.to_integer().to_u32()?;
                    if x > 1000 {
                        return Some(Err("Fattoriale esatto: massimo 1000".into()));
                    }
                    R::from_integer((1..=x).fold(BigInt::one(), |v, x| v * x))
                }
                _ => return None,
            }
        }
    };
    if value.numer().bits() > 16384 || value.denom().bits() > 16384 {
        return Some(Err("Risultato troppo grande".into()));
    }
    Some(Ok(value))
}
fn format(value: &R) -> String {
    if value.is_zero() {
        return "0".into();
    }
    let order =
        value.numer().abs().to_string().len() as i32 - value.denom().to_string().len() as i32;
    if !(-24..=40).contains(&order) {
        let ten = BigInt::from(10);
        let mut mantissa = if order >= 0 {
            value.abs() / R::from_integer(ten.pow(order as u32))
        } else {
            value.abs() * R::from_integer(ten.pow((-order) as u32))
        };
        let mut exponent = order;
        if mantissa < R::one() {
            mantissa *= BigInt::from(10);
            exponent -= 1;
        }
        return format!(
            "{}{}e{exponent:+}",
            if value.is_negative() { "-" } else { "" },
            decimal(&mantissa)
        );
    }
    decimal(value)
}
fn decimal(value: &R) -> String {
    if value.is_zero() {
        return "0".into();
    }
    let negative = value.is_negative();
    let numerator = value.numer().abs();
    let denominator = value.denom();
    let mut whole = (&numerator / denominator).to_string();
    let mut remainder = numerator % denominator;
    let mut fraction = String::new();
    for _ in 0..32 {
        if remainder.is_zero() {
            break;
        }
        remainder *= 10;
        let digit = (&remainder / denominator).to_u8().unwrap_or(0);
        fraction.push((b'0' + digit) as char);
        remainder %= denominator;
    }
    if !remainder.is_zero() && remainder * 2 >= *denominator {
        let combined = format!("{whole}{fraction}");
        let rounded = (BigInt::from_str(&combined).unwrap() + 1u8).to_string();
        let decimals = fraction.len();
        let padded = if rounded.len() <= decimals {
            format!("{:0>width$}", rounded, width = decimals + 1)
        } else {
            rounded
        };
        let cut = padded.len() - decimals;
        whole = padded[..cut].into();
        fraction = padded[cut..].into();
    }
    let fraction = fraction.trim_end_matches('0');
    let result = if fraction.is_empty() {
        whole
    } else {
        format!("{whole}.{fraction}")
    };
    if negative {
        format!("-{result}")
    } else {
        result
    }
}
pub fn expression(ast: &Expr, angle: Angle) -> CalcResult<String> {
    if let Some(value) = exact(ast) {
        return value.map(|v| format(&v));
    }
    ast.eval(angle, &BTreeMap::new()).map(engine::fmt)
}
pub fn evaluate(s: &str, angle: Angle) -> CalcResult<String> {
    expression(&engine::parse(s)?, angle)
}
pub fn internal_expression(ast: &Expr, angle: Angle) -> CalcResult<String> {
    if let Some(value) = exact(ast) {
        return value.map(|v| {
            if v.denom().is_one() {
                v.numer().to_string()
            } else {
                format!("({}/{})", v.numer(), v.denom())
            }
        });
    }
    ast.eval(angle, &BTreeMap::new()).map(engine::fmt)
}
pub fn internal(s: &str, angle: Angle) -> CalcResult<String> {
    internal_expression(&engine::parse(s)?, angle)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn exact_decimal_and_large_integers() {
        assert_eq!(evaluate("1e-100", Angle::Deg).unwrap(), "1e-100");
        assert_eq!(evaluate("1e100", Angle::Deg).unwrap(), "1e+100");
        assert!(evaluate("1e-2147483648", Angle::Deg).is_err());
        assert_eq!(evaluate("0.1+0.2", Angle::Deg).unwrap(), "0.3");
        assert_eq!(
            evaluate("99999999999999999999999999999999+1", Angle::Deg).unwrap(),
            "100000000000000000000000000000000"
        );
        assert_eq!(evaluate("1/3*3", Angle::Deg).unwrap(), "1");
        assert_eq!(evaluate("1/8", Angle::Deg).unwrap(), "0.125");
        assert_eq!(evaluate("-2.5*4", Angle::Deg).unwrap(), "-10");
        assert_eq!(
            evaluate("fact(25)", Angle::Deg).unwrap(),
            "15511210043330985984000000"
        );
        assert!(evaluate("1/0", Angle::Deg).is_err());
    }
}
