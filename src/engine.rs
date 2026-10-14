use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub type CalcResult<T> = Result<T, String>;
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub enum Angle {
    #[default]
    Deg,
    Rad,
    Grad,
}
impl Angle {
    pub fn radians(self, x: f64) -> f64 {
        match self {
            Self::Deg => x.to_radians(),
            Self::Rad => x,
            Self::Grad => x * std::f64::consts::PI / 200.0,
        }
    }
    pub fn from_radians(self, x: f64) -> f64 {
        match self {
            Self::Deg => x.to_degrees(),
            Self::Rad => x,
            Self::Grad => x * 200.0 / std::f64::consts::PI,
        }
    }
}
pub fn finite(v: f64) -> CalcResult<f64> {
    if v.is_finite() {
        Ok(if v == 0.0 { 0.0 } else { v })
    } else {
        Err("Risultato non definito o fuori intervallo".into())
    }
}
pub fn fmt(x: f64) -> String {
    if x == 0.0 {
        return "0".into();
    }
    if x.abs() >= 1e16 || x.abs() < 1e-9 {
        return format!("{x:.12e}");
    }
    let precision = (14.0 - x.abs().log10().floor()).clamp(0.0, 15.0) as usize;
    let s = format!("{x:.precision$}");
    if s.contains('.') {
        s.trim_end_matches('0').trim_end_matches('.').to_owned()
    } else {
        s
    }
}
#[derive(Clone, Debug)]
enum Token {
    Number(String),
    Name(String),
    Op(String),
    Left,
    Right,
    Comma,
    End,
}
#[derive(Clone, Debug)]
pub enum Expr {
    Number(String),
    Variable(String),
    Unary(String, Box<Expr>),
    Binary(String, Box<Expr>, Box<Expr>),
    Call(String, Vec<Expr>),
}
fn lex(input: &str) -> CalcResult<Vec<Token>> {
    if input.len() > 4096 {
        return Err("Espressione troppo lunga".into());
    }
    let text = input
        .replace('×', "*")
        .replace('÷', "/")
        .replace('−', "-")
        .replace('π', "pi");
    let bytes = text.as_bytes();
    let mut i = 0;
    let mut out = vec![];
    while i < bytes.len() {
        let c = bytes[i] as char;
        if c.is_ascii_whitespace() {
            i += 1;
            continue;
        }
        if c.is_ascii_digit() || c == '.' {
            let start = i;
            i += 1;
            if c == '0' && i < bytes.len() && (bytes[i] == b'x' || bytes[i] == b'X') {
                i += 1;
                while i < bytes.len() && (bytes[i] as char).is_ascii_hexdigit() {
                    i += 1
                }
            } else {
                while i < bytes.len() && ((bytes[i] as char).is_ascii_digit() || bytes[i] == b'.') {
                    i += 1
                }
                if i < bytes.len() && (bytes[i] == b'e' || bytes[i] == b'E') {
                    i += 1;
                    if i < bytes.len() && (bytes[i] == b'+' || bytes[i] == b'-') {
                        i += 1
                    }
                    while i < bytes.len() && (bytes[i] as char).is_ascii_digit() {
                        i += 1
                    }
                }
            }
            out.push(Token::Number(text[start..i].into()));
            continue;
        }
        if c.is_ascii_alphabetic() || c == '_' {
            let start = i;
            i += 1;
            while i < bytes.len()
                && ((bytes[i] as char).is_ascii_alphanumeric() || bytes[i] == b'_')
            {
                i += 1
            }
            let name = text[start..i].to_lowercase();
            if [
                "mod", "and", "or", "xor", "nand", "nor", "lsh", "rsh", "rol", "ror",
            ]
            .contains(&name.as_str())
            {
                out.push(Token::Op(name))
            } else {
                out.push(Token::Name(name))
            };
            continue;
        }
        match c {
            '(' => out.push(Token::Left),
            ')' => out.push(Token::Right),
            ',' | ';' => out.push(Token::Comma),
            '+' | '-' | '*' | '/' | '^' | '%' | '!' | '~' | '&' | '|' => {
                out.push(Token::Op(c.to_string()))
            }
            '<' | '>' if i + 1 < bytes.len() && bytes[i + 1] == bytes[i] => {
                out.push(Token::Op(text[i..i + 2].into()));
                i += 1
            }
            _ => return Err(format!("Carattere non riconosciuto: {c}")),
        };
        i += 1;
    }
    out.push(Token::End);
    Ok(out)
}
struct Parser {
    tokens: Vec<Token>,
    index: usize,
    depth: usize,
}
impl Parser {
    fn take(&mut self) -> Token {
        let t = self.tokens[self.index].clone();
        self.index += 1;
        t
    }
    fn peek(&self) -> &Token {
        &self.tokens[self.index]
    }
    fn expr(&mut self, min: u8) -> CalcResult<Expr> {
        self.depth += 1;
        if self.depth > 96 {
            return Err("Troppe parentesi annidate".into());
        }
        let mut left = match self.take() {
            Token::Number(v) => Expr::Number(v),
            Token::Name(n) => {
                if matches!(self.peek(), Token::Left) {
                    self.take();
                    let mut args = vec![];
                    if !matches!(self.peek(), Token::Right) {
                        loop {
                            args.push(self.expr(0)?);
                            if matches!(self.peek(), Token::Comma) {
                                self.take();
                            } else {
                                break;
                            }
                        }
                    }
                    if !matches!(self.take(), Token::Right) {
                        return Err("Manca una parentesi chiusa".into());
                    }
                    Expr::Call(n, args)
                } else {
                    Expr::Variable(n)
                }
            }
            Token::Op(o) if o == "-" || o == "+" || o == "~" => {
                let power = if o == "~" { 35 } else { 25 };
                Expr::Unary(o, Box::new(self.expr(power)?))
            }
            Token::Left => {
                let x = self.expr(0)?;
                if !matches!(self.take(), Token::Right) {
                    return Err("Manca una parentesi chiusa".into());
                }
                x
            }
            _ => return Err("Espressione incompleta".into()),
        };
        loop {
            if matches!(self.peek(),Token::Op(o) if o=="!") {
                if 40 < min {
                    break;
                }
                self.take();
                left = Expr::Call("fact".into(), vec![left]);
                continue;
            }
            let (op, implicit) = match self.peek() {
                Token::Op(o) => (o.clone(), false),
                Token::Left | Token::Name(_) => ("*".into(), true),
                _ => break,
            };
            let (lb, rb) = match op.as_str() {
                "or" | "|" | "nor" => (2, 3),
                "xor" => (4, 5),
                "and" | "&" | "nand" => (6, 7),
                "<<" | ">>" | "lsh" | "rsh" | "rol" | "ror" => (8, 9),
                "+" | "-" => (10, 11),
                "*" | "/" | "%" | "mod" => (20, 21),
                "^" => (30, 30),
                _ => break,
            };
            if lb < min {
                break;
            }
            if !implicit {
                self.take();
            }
            let right = self.expr(rb)?;
            left = Expr::Binary(op, Box::new(left), Box::new(right));
        }
        self.depth -= 1;
        Ok(left)
    }
}
pub fn parse(input: &str) -> CalcResult<Expr> {
    let mut p = Parser {
        tokens: lex(input)?,
        index: 0,
        depth: 0,
    };
    let e = p.expr(0)?;
    if !matches!(p.peek(), Token::End) {
        return Err("Controlla operatori e parentesi".into());
    }
    Ok(e)
}
impl Expr {
    pub fn eval(&self, angle: Angle, vars: &BTreeMap<String, f64>) -> CalcResult<f64> {
        finite(match self {
            Self::Number(s) => {
                if s.starts_with("0x") {
                    u64::from_str_radix(&s[2..], 16)
                        .map(|x| x as f64)
                        .map_err(|_| "Numero non valido")?
                } else {
                    s.parse().map_err(|_| "Numero non valido")?
                }
            }
            Self::Variable(n) => match n.as_str() {
                "pi" => std::f64::consts::PI,
                "e" => std::f64::consts::E,
                _ => *vars
                    .get(n)
                    .ok_or_else(|| format!("Variabile non definita: {n}"))?,
            },
            Self::Unary(o, a) => {
                let a = a.eval(angle, vars)?;
                match o.as_str() {
                    "-" => -a,
                    "+" => a,
                    _ => return Err("Operatore riservato al programmatore".into()),
                }
            }
            Self::Binary(o, a, b) => {
                let a = a.eval(angle, vars)?;
                let b = b.eval(angle, vars)?;
                match o.as_str() {
                    "+" => a + b,
                    "-" => a - b,
                    "*" => a * b,
                    "/" => {
                        if b == 0.0 {
                            return Err("Impossibile dividere per zero".into());
                        }
                        a / b
                    }
                    "^" => {
                        if a == 0.0 && b == 0.0 {
                            return Err("0 elevato a 0 non è definito".into());
                        }
                        a.powf(b)
                    }
                    "mod" | "%" => {
                        if b == 0.0 {
                            return Err("Impossibile dividere per zero".into());
                        }
                        a % b
                    }
                    _ => return Err("Operatore riservato al programmatore".into()),
                }
            }
            Self::Call(n, args) => {
                let v = args
                    .iter()
                    .map(|a| a.eval(angle, vars))
                    .collect::<CalcResult<Vec<_>>>()?;
                call(n, &v, angle)?
            }
        })
    }
}
fn call(n: &str, v: &[f64], angle: Angle) -> CalcResult<f64> {
    let expected = if ["root", "logy", "pow", "min", "max"].contains(&n) {
        2
    } else {
        1
    };
    if v.len() != expected {
        return Err(format!("{n}: servono {expected} argomenti"));
    }
    let x = v[0];
    let r = angle.radians(x);
    let inverse = |x| angle.from_radians(x);
    finite(match n {
        "sin" => r.sin(),
        "cos" => r.cos(),
        "tan" => {
            if r.cos().abs() < 1e-15 {
                return Err("Tangente non definita".into());
            }
            r.tan()
        }
        "asin" => inverse(x.asin()),
        "acos" => inverse(x.acos()),
        "atan" => inverse(x.atan()),
        "sec" => {
            if r.cos().abs() < 1e-15 {
                return Err("Secante non definita".into());
            }
            1.0 / r.cos()
        }
        "csc" => {
            if r.sin().abs() < 1e-15 {
                return Err("Cosecante non definita".into());
            }
            1.0 / r.sin()
        }
        "cot" => {
            if r.sin().abs() < 1e-15 {
                return Err("Cotangente non definita".into());
            }
            r.cos() / r.sin()
        }
        "asec" => inverse((1.0 / x).acos()),
        "acsc" => inverse((1.0 / x).asin()),
        "acot" => inverse((1.0 / x).atan()),
        "sinh" => x.sinh(),
        "cosh" => x.cosh(),
        "tanh" => x.tanh(),
        "asinh" => x.asinh(),
        "acosh" => x.acosh(),
        "atanh" => x.atanh(),
        "sech" => 1.0 / x.cosh(),
        "csch" => 1.0 / x.sinh(),
        "coth" => 1.0 / x.tanh(),
        "asech" => (1.0 / x).acosh(),
        "acsch" => (1.0 / x).asinh(),
        "acoth" => (1.0 / x).atanh(),
        "sqrt" => x.sqrt(),
        "sq" => x * x,
        "cube" => x * x * x,
        "cbrt" => x.cbrt(),
        "abs" => x.abs(),
        "floor" => x.floor(),
        "ceil" => x.ceil(),
        "round" => x.round(),
        "ln" => x.ln(),
        "log" | "log10" => x.log10(),
        "exp" => x.exp(),
        "pow10" => 10.0_f64.powf(x),
        "pow2" => 2.0_f64.powf(x),
        "inv" => 1.0 / x,
        "fact" => {
            if x < 0.0 || x.fract() != 0.0 || x > 170.0 {
                return Err("Fattoriale: intero da 0 a 170".into());
            }
            (1..=x as u32).fold(1.0, |a, b| a * b as f64)
        }
        "pow" => x.powf(v[1]),
        "root" => {
            if v[1] == 0.0 {
                return Err("Indice della radice nullo".into());
            }
            if x < 0.0 && v[1].fract() == 0.0 && (v[1] as i64) % 2 != 0 {
                -(-x).powf(1.0 / v[1])
            } else {
                x.powf(1.0 / v[1])
            }
        }
        "logy" => x.log(v[1]),
        "min" => x.min(v[1]),
        "max" => x.max(v[1]),
        "dms" => {
            let a = x.abs();
            let d = a.floor();
            let m = ((a - d) * 60.0).floor();
            x.signum() * (d + m / 100.0 + ((a - d) * 60.0 - m) * 60.0 / 10000.0)
        }
        "degrees" => {
            let a = x.abs();
            let d = a.floor();
            let m = ((a - d) * 100.0 + 1e-9).floor();
            let sec = ((a - d) * 100.0 - m) * 100.0;
            if m >= 60.0 || sec >= 60.0 + 1e-6 {
                return Err("Minuti e secondi devono essere inferiori a 60".into());
            }
            x.signum() * (d + m / 60.0 + sec / 3600.0)
        }
        _ => return Err(format!("Funzione non riconosciuta: {n}")),
    })
}
pub fn evaluate(s: &str, angle: Angle) -> CalcResult<f64> {
    parse(s)?.eval(angle, &BTreeMap::new())
}

#[derive(Clone, Serialize, Deserialize)]
pub struct History {
    pub expression: String,
    pub result: String,
}
#[derive(Default)]
pub struct Calculator {
    pub input: String,
    pub display: String,
    pub error: Option<String>,
    pub angle: Angle,
    pub scientific: bool,
    pub exponential: bool,
    pub fresh: bool,
    pub history: Vec<History>,
    pub memory: Vec<String>,
    repeat: Option<(String, String)>,
    replace_operand: bool,
}
impl Calculator {
    pub fn new() -> Self {
        Self {
            display: "0".into(),
            fresh: true,
            ..Self::default()
        }
    }
    pub fn clear(&mut self) {
        self.input.clear();
        self.display = "0".into();
        self.error = None;
        self.fresh = true;
        self.replace_operand = false;
        self.repeat = None
    }
    #[cfg(test)]
    pub fn value(&self) -> CalcResult<f64> {
        if self.input.is_empty() {
            evaluate(&self.display, self.angle)
        } else {
            evaluate(&self.input, self.angle)
        }
    }
    pub fn set_value(&mut self, x: f64) {
        self.input = fmt(x);
        self.display = fmt(x);
        self.fresh = true;
        self.error = None;
        self.repeat = None
    }
    pub fn set_text(&mut self, text: &str) {
        match crate::precise::evaluate(text, self.angle) {
            Ok(result) => {
                self.input =
                    crate::precise::internal(text, self.angle).unwrap_or_else(|_| result.clone());
                self.display = result;
                self.fresh = true;
                self.replace_operand = false;
                self.error = None;
                self.repeat = None;
            }
            Err(e) => self.error = Some(e),
        }
    }
    pub fn digit(&mut self, t: &str) {
        if self.replace_operand && !self.fresh {
            self.input.truncate(self.operand_start());
        }
        self.replace_operand = false;
        if self.fresh || self.error.is_some() {
            self.input.clear();
            self.fresh = false;
            self.error = None;
            self.repeat = None
        }
        self.input.push_str(t);
        self.preview()
    }
    pub fn dot(&mut self) {
        if self.replace_operand && !self.fresh {
            self.input.truncate(self.operand_start());
        }
        self.replace_operand = false;
        if self.fresh {
            self.input.clear();
            self.fresh = false
        }
        let start = self.operand_start();
        if !self.input[start..].contains('.') {
            if self.input[start..].is_empty() {
                self.input.push('0')
            }
            self.input.push('.')
        }
        self.preview()
    }
    pub fn operator(&mut self, op: &str) {
        self.replace_operand = false;
        self.error = None;
        self.repeat = None;
        if self.input.is_empty() {
            self.input = self.display.clone()
        }
        while self.input.ends_with(' ') {
            self.input.pop();
        }
        if let Some(last) = self.input.chars().last() {
            if "+-*/^".contains(last) {
                self.input.pop();
            }
        }
        if !self.scientific {
            if let Ok(v) = crate::precise::internal(&self.input, self.angle) {
                self.input = v.clone();
                self.display = crate::precise::evaluate(&v, self.angle).unwrap_or(v)
            }
        }
        self.input.push_str(op);
        self.fresh = false;
    }
    fn operand_start(&self) -> usize {
        let mut depth = 0;
        for (i, c) in self.input.char_indices().rev() {
            match c {
                ')' => depth += 1,
                '(' => {
                    if depth > 0 {
                        depth -= 1
                    }
                }
                '+' | '-' | '*' | '/' | '^' if depth == 0 => {
                    if i > 0 && !matches!(self.input.as_bytes()[i - 1], b'e' | b'E') {
                        return i + c.len_utf8();
                    }
                }
                _ => {}
            }
        }
        0
    }
    pub fn ce(&mut self) {
        let start = self.operand_start();
        self.input.truncate(start);
        self.input.push('0');
        self.replace_operand = true;
        self.error = None;
        self.fresh = false;
        self.preview()
    }
    pub fn back(&mut self) {
        self.replace_operand = false;
        if self.fresh {
            return;
        }
        self.input.pop();
        self.error = None;
        self.preview()
    }
    pub fn unary(&mut self, name: &str) {
        if self.input.is_empty() {
            self.input = self.display.clone()
        }
        let start = self.operand_start();
        let operand = self.input[start..].to_owned();
        if operand.is_empty() {
            return;
        }
        let s = if name == "neg" {
            format!("-({operand})")
        } else {
            format!("{name}({operand})")
        };
        let expression = format!("{}{}", &self.input[..start], s);
        match crate::precise::evaluate(&expression, self.angle) {
            Ok(v) => {
                self.input = expression;
                self.display = v;
                self.fresh = false;
                self.replace_operand = true;
                self.error = None
            }
            Err(e) => self.error = Some(e),
        }
    }
    pub fn percent(&mut self) {
        let start = self.operand_start();
        let part = self.input[start..].to_owned();
        if part.is_empty() {
            return;
        }
        let mut expression = format!("({part})/100");
        if start > 0 && matches!(self.input.as_bytes()[start - 1], b'+' | b'-') {
            expression = format!("({})*({part})/100", &self.input[..start - 1]);
        }
        let Ok(v) = crate::precise::internal(&expression, self.angle) else {
            return;
        };
        self.input.truncate(start);
        self.input.push_str(&v);
        self.replace_operand = true;
        self.preview()
    }
    pub fn equals(&mut self) {
        let mut expression = if self.input.is_empty() {
            self.display.clone()
        } else {
            self.input.clone()
        };
        if self.fresh {
            if let Some((op, rhs)) = &self.repeat {
                expression = format!("({}){}({})", self.input, op, rhs)
            }
        }
        let balance = expression.chars().fold(0i32, |n, c| {
            n + if c == '(' {
                1
            } else if c == ')' {
                -1
            } else {
                0
            }
        });
        if balance > 0 && balance <= 96 {
            expression.push_str(&")".repeat(balance as usize));
        }
        match parse(&expression).and_then(|ast| {
            if let Expr::Binary(op, _, rhs) = &ast {
                if let Ok(v) = crate::precise::internal_expression(rhs, self.angle) {
                    self.repeat = Some((op.clone(), v))
                }
            }
            crate::precise::expression(&ast, self.angle)
        }) {
            Ok(v) => {
                let result = v;
                self.replace_operand = false;
                self.history.push(History {
                    expression: expression.clone(),
                    result: result.clone(),
                });
                if self.history.len() > 200 {
                    self.history.remove(0);
                }
                self.input = crate::precise::internal(&expression, self.angle)
                    .unwrap_or_else(|_| result.clone());
                self.display = result;
                self.fresh = true;
                self.error = None;
                self.preview();
            }
            Err(e) => self.error = Some(e),
        }
    }
    pub fn constant(&mut self, name: &str) {
        let start = if self.fresh { 0 } else { self.operand_start() };
        self.input.truncate(start);
        self.input.push_str(name);
        self.fresh = false;
        self.replace_operand = true;
        self.error = None;
        self.preview();
    }
    pub fn enter_exponent(&mut self) {
        if self.input.is_empty() {
            self.input = "1".into()
        }
        let start = self.operand_start();
        let operand = self.input[start..].to_string();
        if let Ok(value) = crate::precise::evaluate(&operand, self.angle) {
            if value.contains('e') {
                return;
            }
            self.input.truncate(start);
            self.input.push_str(&value);
            self.input.push('e');
            self.fresh = false;
            self.replace_operand = false;
        }
    }
    pub fn start_binary_function(&mut self, name: &str) {
        if self.input.is_empty() {
            self.input = "0".into()
        }
        let start = self.operand_start();
        let operand = self.input[start..].to_string();
        if operand.is_empty() {
            self.error = Some("Inserisci un operando".into());
            return;
        }
        self.input = format!("{}{name}({operand},", &self.input[..start]);
        self.fresh = false;
        self.replace_operand = false;
        self.error = None;
    }
    pub fn preview(&mut self) {
        if self.input.is_empty() {
            self.display = "0".into()
        } else if let Ok(v) = crate::precise::evaluate(&self.input, self.angle) {
            self.display = if self.exponential {
                v.parse::<f64>()
                    .ok()
                    .filter(|x| x.is_finite())
                    .map(|x| format!("{x:.10e}"))
                    .unwrap_or(v)
            } else {
                v
            }
        }
    }
    pub fn memory_action(&mut self, action: &str) {
        let operand = if self.input.is_empty() {
            "0".to_string()
        } else {
            self.input.clone()
        };
        match action {
            "MC" => self.memory.clear(),
            "MR" => {
                if let Some(x) = self.memory.first().cloned() {
                    self.set_text(&x)
                }
            }
            "MS" => {
                if let Ok(x) = crate::precise::internal(&operand, self.angle) {
                    self.memory.insert(0, x);
                    self.memory.truncate(50)
                }
            }
            "M+" | "M−" => {
                if let Ok(x) = crate::precise::internal(&operand, self.angle) {
                    let op = if action == "M+" { "+" } else { "-" };
                    let previous = self.memory.first().cloned().unwrap_or("0".into());
                    if let Ok(value) =
                        crate::precise::internal(&format!("({previous}){op}({x})"), self.angle)
                    {
                        if self.memory.is_empty() {
                            self.memory.push(value)
                        } else {
                            self.memory[0] = value
                        }
                    }
                }
            }
            _ => {}
        }
    }
}

#[derive(Default)]
pub struct Programmer {
    pub input: String,
    pub radix: u32,
    pub bits: u32,
    pub result: u64,
    pub error: Option<String>,
    pub fresh: bool,
    pub carry: bool,
    pub memory: Vec<u64>,
}
impl Programmer {
    pub fn new() -> Self {
        Self {
            radix: 10,
            bits: 64,
            fresh: true,
            ..Self::default()
        }
    }
    pub fn mask(&self) -> u64 {
        if self.bits == 64 {
            u64::MAX
        } else {
            (1u64 << self.bits) - 1
        }
    }
    pub fn signed(&self, x: u64) -> i64 {
        let shift = 64 - self.bits;
        ((x << shift) as i64) >> shift
    }
    pub fn text(&self, radix: u32) -> String {
        match radix {
            16 => format!("{:X}", self.result),
            8 => format!("{:o}", self.result),
            2 => format!("{:b}", self.result),
            _ => self.signed(self.result).to_string(),
        }
    }
    pub fn set(&mut self, x: u64) {
        self.result = x & self.mask();
        self.input = self.text(self.radix);
        self.fresh = true;
        self.error = None
    }
    pub fn digit(&mut self, c: char) {
        if c.to_digit(self.radix).is_none() {
            return;
        }
        if self.fresh {
            self.input.clear();
            self.fresh = false
        }
        self.input.push(c);
        self.error = None;
        if let Ok(x) = self.eval() {
            self.result = x
        }
    }
    pub fn eval(&self) -> CalcResult<u64> {
        // Prefix numeric words before the shared lexer: 1A is one hexadecimal
        // operand, never the implicit multiplication 1 * A.
        let mut normalized = String::new();
        let mut word = String::new();
        let flush = |word: &mut String, text: &mut String| -> CalcResult<()> {
            if word.is_empty() {
                return Ok(());
            }
            if word.chars().all(|c| c.is_digit(self.radix)) {
                let value = u64::from_str_radix(word, self.radix)
                    .map_err(|_| "Numero troppo grande per 64 bit".to_string())?;
                text.push_str(&format!("0x{value:X}"));
            } else {
                text.push_str(word);
            }
            word.clear();
            Ok(())
        };
        for c in self.input.chars() {
            if c.is_ascii_alphanumeric() {
                word.push(c)
            } else {
                flush(&mut word, &mut normalized)?;
                normalized.push(c);
            }
        }
        flush(&mut word, &mut normalized)?;
        let ast = parse(&normalized)?;
        self.eval_ast(&ast).map(|x| x & self.mask())
    }
    fn eval_ast(&self, e: &Expr) -> CalcResult<u64> {
        let mask = self.mask();
        Ok(match e {
            Expr::Number(s) | Expr::Variable(s) => {
                let (digits, base) = if s.starts_with("0x") {
                    (&s[2..], 16)
                } else {
                    (s.as_str(), self.radix)
                };
                u64::from_str_radix(digits, base)
                    .map_err(|_| "Numero non valido o troppo grande per 64 bit")?
                    & mask
            }
            Expr::Unary(o, a) => {
                let x = self.eval_ast(a)?;
                (match o.as_str() {
                    "-" => x.wrapping_neg(),
                    "~" => !x,
                    _ => x,
                }) & mask
            }
            Expr::Binary(o, a, b) => {
                let a = self.eval_ast(a)?;
                let b = self.eval_ast(b)?;
                let shift = (b % self.bits as u64) as u32;
                (match o.as_str() {
                    "+" => a.wrapping_add(b),
                    "-" => a.wrapping_sub(b),
                    "*" => a.wrapping_mul(b),
                    "/" | "%" | "mod" => {
                        let sa = self.signed(a);
                        let sb = self.signed(b);
                        if sb == 0 {
                            return Err("Impossibile dividere per zero".into());
                        }
                        if o == "/" {
                            sa.wrapping_div(sb) as u64
                        } else {
                            sa.wrapping_rem(sb) as u64
                        }
                    }
                    "and" | "&" => a & b,
                    "or" | "|" => a | b,
                    "xor" => a ^ b,
                    "nand" => !(a & b),
                    "nor" => !(a | b),
                    "lsh" | "<<" => {
                        if b >= self.bits as u64 {
                            0
                        } else {
                            a << b
                        }
                    }
                    "rsh" | ">>" => {
                        if b >= self.bits as u64 {
                            if self.signed(a) < 0 {
                                mask
                            } else {
                                0
                            }
                        } else {
                            (self.signed(a) >> b) as u64
                        }
                    }
                    "rol" => {
                        if shift == 0 {
                            a
                        } else {
                            (a << shift) | (a >> (self.bits - shift))
                        }
                    }
                    "ror" => {
                        if shift == 0 {
                            a
                        } else {
                            (a >> shift) | (a << (self.bits - shift))
                        }
                    }
                    _ => return Err("Operatore intero non supportato".into()),
                }) & mask
            }
            _ => return Err("Funzione non disponibile in modalità Programmatore".into()),
        })
    }
    pub fn operator(&mut self, op: &str) {
        if self.input.is_empty() {
            self.input = self.text(self.radix)
        }
        self.input.push_str(&format!(" {op} "));
        self.fresh = false;
        self.error = None
    }
    pub fn equals(&mut self) {
        match self.eval() {
            Ok(v) => self.set(v),
            Err(e) => self.error = Some(e),
        }
    }
    pub fn ce(&mut self) {
        let index = self.input.rfind(' ').map(|i| i + 1).unwrap_or(0);
        self.input.truncate(index);
        self.input.push('0');
        self.error = None;
        self.fresh = false;
        if let Ok(v) = self.eval() {
            self.result = v
        }
    }
    pub fn back(&mut self) {
        if self.fresh {
            return;
        }
        self.input.pop();
        self.error = None;
        if self.input.is_empty() {
            self.result = 0
        } else if let Ok(v) = self.eval() {
            self.result = v
        }
    }
    pub fn flip(&mut self, bit: u32) {
        self.set(self.result ^ (1u64 << bit))
    }
    pub fn switch(&mut self, radix: u32, bits: u32) {
        self.radix = radix;
        self.bits = bits;
        self.set(self.result)
    }
    pub fn shift(&mut self, op: &str) {
        let x = self.result;
        let top = 1u64 << (self.bits - 1);
        let v = match op {
            "NOT" => !x,
            "LSR" => x >> 1,
            "ASR" => (self.signed(x) >> 1) as u64,
            "LSH" => {
                self.carry = x & top != 0;
                x << 1
            }
            "ROL" => (x << 1) | ((x & top) >> (self.bits - 1)),
            "ROR" => (x >> 1) | ((x & 1) << (self.bits - 1)),
            "ROLC" => {
                let c = self.carry as u64;
                self.carry = x & top != 0;
                (x << 1) | c
            }
            "RORC" => {
                let c = self.carry as u64;
                self.carry = x & 1 != 0;
                (x >> 1) | (c << (self.bits - 1))
            }
            _ => x,
        };
        self.set(v)
    }
    pub fn memory_action(&mut self, action: &str) {
        match action {
            "MC" => self.memory.clear(),
            "MR" => {
                if let Some(x) = self.memory.first().copied() {
                    self.set(x)
                }
            }
            "MS" => {
                self.memory.insert(0, self.result);
                self.memory.truncate(50);
            }
            "M+" | "M−" => {
                let previous = self.memory.first().copied().unwrap_or(0);
                let x = if action == "M+" {
                    previous.wrapping_add(self.result)
                } else {
                    previous.wrapping_sub(self.result)
                } & self.mask();
                if self.memory.is_empty() {
                    self.memory.push(x)
                } else {
                    self.memory[0] = x
                }
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn e(s: &str) -> f64 {
        evaluate(s, Angle::Deg).unwrap()
    }
    #[test]
    fn precedence() {
        assert_eq!(e("2+3*4"), 14.0);
        assert_eq!(e("(2+3)*4"), 20.0);
        assert_eq!(e("-2^2"), -4.0);
        assert_eq!(e("2^3^2"), 512.0);
        assert!((e("sin(30)") - 0.5).abs() < 1e-12);
        assert_eq!(e("root(-8,3)"), -2.0);
        assert_eq!(e("5!"), 120.0)
    }
    #[test]
    fn invalid() {
        for s in [
            "1/0",
            "sqrt(-1)",
            "fact(1.5)",
            "0^0",
            "2+",
            "((2)",
            "tan(90)",
        ] {
            assert!(evaluate(s, Angle::Deg).is_err(), "{s}")
        }
    }
    #[test]
    fn standard_percent_ce_repeat() {
        let mut c = Calculator::new();
        c.digit("200");
        c.operator("+");
        c.digit("10");
        c.percent();
        c.equals();
        assert_eq!(c.display, "220");
        c.equals();
        assert_eq!(c.display, "240");
        c.clear();
        c.digit("5");
        c.operator("+");
        c.digit("9");
        c.ce();
        c.digit("3");
        c.equals();
        assert_eq!(c.value().unwrap(), 8.0)
    }
    #[test]
    fn standard_vs_scientific() {
        let mut c = Calculator::new();
        c.digit("2");
        c.operator("+");
        c.digit("3");
        c.operator("*");
        c.digit("4");
        c.equals();
        assert_eq!(c.display, "20");
        c.clear();
        c.scientific = true;
        c.digit("2+3*4");
        c.equals();
        assert_eq!(c.display, "14")
    }
    #[test]
    fn exact_bits() {
        let mut p = Programmer::new();
        p.radix = 16;
        p.input = "FFFFFFFFFFFFFFFF".into();
        assert_eq!(p.eval().unwrap(), u64::MAX);
        p.equals();
        assert_eq!(p.text(10), "-1");
        p.switch(16, 8);
        p.input = "FF + 1".into();
        p.equals();
        assert_eq!(p.result, 0);
        p.set(128);
        p.shift("ASR");
        assert_eq!(p.result, 192);
        p.set(128);
        p.shift("ROLC");
        assert!(p.carry);
        assert_eq!(p.result, 0);
        p.shift("ROLC");
        assert_eq!(p.result, 1)
    }
    #[test]
    fn all_scientific_families() {
        for expression in [
            "sq(3)",
            "cube(-2)",
            "inv(8)",
            "abs(-2)",
            "floor(2.9)",
            "ceil(2.1)",
            "round(2.4)",
            "log(100)",
            "ln(e)",
            "pow10(2)",
            "pow2(3)",
            "exp(0)",
            "sinh(1)",
            "cosh(1)",
            "tanh(1)",
            "asinh(1)",
            "acosh(2)",
            "atanh(0.5)",
            "sech(1)",
            "csch(1)",
            "coth(1)",
            "asech(0.5)",
            "acsch(1)",
            "acoth(2)",
            "sec(60)",
            "csc(30)",
            "cot(45)",
            "asec(2)",
            "acsc(2)",
            "acot(1)",
            "pow(2,3)",
            "logy(8,2)",
            "degrees(12.3045)",
            "dms(12.5)",
        ] {
            assert!(evaluate(expression, Angle::Deg).is_ok(), "{expression}");
        }
        assert!((evaluate("sin(pi/2)", Angle::Rad).unwrap() - 1.0).abs() < 1e-12);
        assert!((evaluate("sin(100)", Angle::Grad).unwrap() - 1.0).abs() < 1e-12);
    }
    #[test]
    fn unary_replaces_operand_and_memory() {
        let mut exp = Calculator::new();
        exp.digit("2");
        exp.enter_exponent();
        exp.digit("3");
        exp.equals();
        assert_eq!(exp.display, "2000");
        exp.exponential = true;
        exp.equals();
        assert!(exp.display.contains('e'));
        let mut root = Calculator::new();
        root.scientific = true;
        root.digit("2+27");
        root.start_binary_function("root");
        root.digit("3");
        root.equals();
        assert_eq!(root.display, "5");
        let mut zero = Calculator::new();
        zero.memory_action("MS");
        assert_eq!(zero.memory.len(), 1);
        let mut exact = Calculator::new();
        exact.digit("1");
        exact.operator("/");
        exact.digit("3");
        exact.equals();
        exact.memory_action("MS");
        exact.clear();
        exact.memory_action("MR");
        exact.operator("*");
        exact.digit("3");
        exact.equals();
        assert_eq!(exact.display, "1");
        let mut c = Calculator::new();
        c.digit("5");
        c.operator("+");
        c.digit("9");
        c.unary("sqrt");
        c.digit("2");
        c.equals();
        assert_eq!(c.display, "7");
        c.memory_action("MS");
        c.set_value(3.0);
        c.memory_action("M+");
        c.memory_action("MR");
        assert_eq!(c.display, "10");
        c.memory_action("MC");
        assert!(c.memory.is_empty());
        c.clear();
        c.digit("200");
        c.operator("*");
        c.digit("10");
        c.percent();
        c.equals();
        assert_eq!(c.display, "20");
    }
    #[test]
    fn hex_and_integer_logic() {
        let mut p = Programmer::new();
        p.radix = 16;
        for (s, want) in [
            ("1A + B", 37),
            ("FF and F0", 240),
            ("FF xor F0", 15),
            ("1 lsh 3", 8),
            ("80 ror 1", 64),
        ] {
            p.input = s.into();
            assert_eq!(p.eval().unwrap(), want, "{s}");
        }
        p.set(u64::MAX);
        p.memory_action("MS");
        p.set(0);
        p.memory_action("MR");
        assert_eq!(p.result, u64::MAX);
    }
}
