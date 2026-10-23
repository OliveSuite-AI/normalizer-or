//! Small, conservative parser and Odia renderer for elementary algebra.
use crate::{SegmentKind, numerals};

#[derive(Debug)]
enum Expr {
    Number(u64),
    Variable(char),
    Text(char),
    Unary(char, Box<Expr>),
    Binary(char, Box<Expr>, Box<Expr>),
    Power(Box<Expr>, Box<Expr>),
    Group(Box<Expr>),
}

struct Parser {
    chars: Vec<char>,
    at: usize,
}

impl Parser {
    fn peek(&self) -> Option<char> {
        self.chars.get(self.at).copied()
    }
    fn eat(&mut self, c: char) -> bool {
        if self.peek() == Some(c) {
            self.at += 1;
            true
        } else {
            false
        }
    }
    fn expression(&mut self) -> Option<Expr> {
        let mut left = self.product()?;
        while let Some(op @ ('+' | '-')) = self.peek() {
            self.at += 1;
            let right = self.product()?;
            left = Expr::Binary(op, Box::new(left), Box::new(right));
        }
        Some(left)
    }
    fn product(&mut self) -> Option<Expr> {
        let mut left = self.unary()?;
        loop {
            let op = match self.peek() {
                Some('*' | '×') => {
                    self.at += 1;
                    '*'
                }
                Some('/') => {
                    self.at += 1;
                    '/'
                }
                Some(c)
                    if c.is_ascii_alphabetic()
                        || c.is_ascii_digit()
                        || c == '('
                        || text_meaning(c).is_some() =>
                {
                    '*'
                }
                _ => break,
            };
            let right = self.unary()?;
            left = Expr::Binary(op, Box::new(left), Box::new(right));
        }
        Some(left)
    }
    fn unary(&mut self) -> Option<Expr> {
        if self.eat('+') {
            return Some(Expr::Unary('+', Box::new(self.unary()?)));
        }
        if self.eat('-') || self.eat('−') {
            return Some(Expr::Unary('-', Box::new(self.unary()?)));
        }
        let mut x = self.atom()?;
        loop {
            if self.eat('^') {
                let power = if self.eat('{') {
                    let e = self.expression()?;
                    if !self.eat('}') {
                        return None;
                    }
                    e
                } else {
                    self.atom()?
                };
                x = Expr::Power(Box::new(x), Box::new(power));
            } else if let Some(p) = self.peek().and_then(|c| match c {
                '²' => Some(2),
                '³' => Some(3),
                _ => None,
            }) {
                self.at += 1;
                x = Expr::Power(Box::new(x), Box::new(Expr::Number(p)));
            } else {
                break;
            }
        }
        Some(x)
    }
    fn atom(&mut self) -> Option<Expr> {
        if self.eat('(') {
            let e = self.expression()?;
            if !self.eat(')') {
                return None;
            }
            return Some(Expr::Group(Box::new(e)));
        }
        let c = self.peek()?;
        if text_meaning(c).is_some() {
            self.at += 1;
            return Some(Expr::Text(c));
        }
        if c.is_ascii_digit() {
            let mut n = 0u64;
            while let Some(c) = self.peek().filter(|c| c.is_ascii_digit()) {
                n = n.checked_mul(10)?.checked_add(c.to_digit(10)? as u64)?;
                if n >= numerals::MAGNITUDE_LIMIT {
                    return None;
                }
                self.at += 1;
            }
            return Some(Expr::Number(n));
        }
        if c.is_ascii_alphabetic() {
            self.at += 1;
            return Some(Expr::Variable(c.to_ascii_lowercase()));
        }
        None
    }
}

fn latex(source: &str) -> Option<String> {
    latex_inner(source, 0)
}

fn latex_inner(source: &str, depth: usize) -> Option<String> {
    if depth > 64 || source.len() > 4096 {
        return None;
    }
    let mut s = source.trim();
    if let Some((open, close)) = [("$$", "$$"), ("\\[", "\\]"), ("\\(", "\\)")]
        .into_iter()
        .find(|(open, close)| s.starts_with(open) && s.ends_with(close))
    {
        s = &s[open.len()..s.len() - close.len()];
    } else if s.starts_with('$') && s.ends_with('$') {
        s = &s[1..s.len() - 1];
    }
    let mut out = String::new();
    let mut rest = s;
    while !rest.is_empty() {
        if let Some(tail) = rest.strip_prefix("\\left") {
            rest = tail;
            continue;
        }
        if let Some(tail) = rest.strip_prefix("\\right") {
            rest = tail;
            continue;
        }
        if let Some(tail) = rest.strip_prefix("\\cdot") {
            out.push('*');
            rest = tail;
            continue;
        }
        if let Some(tail) = rest.strip_prefix("\\times") {
            out.push('*');
            rest = tail;
            continue;
        }
        if let Some(tail) = rest.strip_prefix("\\div") {
            out.push('/');
            rest = tail;
            continue;
        }
        if let Some(tail) = rest.strip_prefix("\\frac") {
            let (a, r) = braced(tail)?;
            let (b, r) = braced(r)?;
            out.push('(');
            out.push_str(&latex_inner(a, depth + 1)?);
            out.push_str(")/(");
            out.push_str(&latex_inner(b, depth + 1)?);
            out.push(')');
            rest = r;
            continue;
        }
        if let Some(tail) = rest.strip_prefix("\\boxed") {
            let (body, tail) = braced(tail)?;
            out.push_str(&latex_inner(body, depth + 1)?);
            rest = tail;
            continue;
        }
        if let Some(tail) = rest.strip_prefix("\\text") {
            let (label, tail) = braced(tail)?;
            out.push(text_marker(label.trim())?);
            rest = tail;
            continue;
        }
        let c = rest.chars().next()?;
        if c.is_whitespace() {
            if matches!(c, '\n' | '\r') && !out.is_empty() && !out.ends_with(';') {
                out.push(';');
            }
            rest = &rest[c.len_utf8()..];
            continue;
        }
        out.push(match c {
            '−' => '-',
            '×' => '*',
            '÷' => '/',
            _ => c,
        });
        rest = &rest[c.len_utf8()..];
    }
    Some(out.trim_matches(';').to_owned())
}
fn braced(s: &str) -> Option<(&str, &str)> {
    let s = s.trim_start();
    let s = s.strip_prefix('{')?;
    let mut depth = 1;
    for (i, c) in s.char_indices() {
        if c == '{' {
            depth += 1
        } else if c == '}' {
            depth -= 1;
            if depth == 0 {
                return Some((&s[..i], &s[i + 1..]));
            }
        }
    }
    None
}

fn variable(c: char) -> Option<&'static str> {
    Some(match c {
        'a' => "ଏ",
        'b' => "ବି",
        'c' => "ସି",
        'd' => "ଡି",
        'e' => "ଇ",
        'f' => "ଏଫ୍",
        'g' => "ଜି",
        'h' => "ଏଚ୍",
        'i' => "ଆଇ",
        'j' => "ଜେ",
        'k' => "କେ",
        'l' => "ଏଲ୍",
        'm' => "ଏମ୍",
        'n' => "ଏନ୍",
        'o' => "ଓ",
        'p' => "ପି",
        'q' => "କ୍ୟୁ",
        'r' => "ଆର୍",
        's' => "ଏସ୍",
        't' => "ଟି",
        'u' => "ୟୁ",
        'v' => "ଭି",
        'w' => "ଡବ୍ଲ୍ୟୁ",
        'x' => "ଏକ୍ସ",
        'y' => "ୱାଇ",
        'z' => "ଜେଡ୍",
        _ => return None,
    })
}
fn text_marker(label: &str) -> Option<char> {
    Some(match label {
        "Area" => '\u{e000}',
        "length" => '\u{e001}',
        "breadth" => '\u{e002}',
        "Perimeter" => '\u{e003}',
        "cm" => '\u{e004}',
        "mm" => '\u{e005}',
        "m" => '\u{e006}',
        "km" => '\u{e007}',
        _ => return None,
    })
}
fn text_meaning(marker: char) -> Option<&'static str> {
    Some(match marker {
        '\u{e000}' => "କ୍ଷେତ୍ରଫଳ",
        '\u{e001}' => "ଲମ୍ବ",
        '\u{e002}' => "ପ୍ରସ୍ଥ",
        '\u{e003}' => "ପରିଧି",
        '\u{e004}' => "ସେଣ୍ଟିମିଟର",
        '\u{e005}' => "ମିଲିମିଟର",
        '\u{e006}' => "ମିଟର",
        '\u{e007}' => "କିଲୋମିଟର",
        _ => return None,
    })
}
fn unit_reading(expr: &Expr) -> Option<String> {
    if let Expr::Text(unit) = expr {
        return text_meaning(*unit).map(str::to_owned);
    }
    if let Expr::Power(base, exponent) = expr
        && let Expr::Text(unit) = base.as_ref()
        && matches!(exponent.as_ref(), Expr::Number(2))
    {
        return text_meaning(*unit).map(|name| format!("ବର୍ଗ {name}"));
    }
    None
}
fn render(e: &Expr) -> Option<String> {
    Some(match e {
        Expr::Number(n) => numerals::cardinal(*n),
        Expr::Variable(c) => variable(*c)?.to_owned(),
        Expr::Text(marker) => text_meaning(*marker)?.to_owned(),
        Expr::Unary(op, x) => format!(
            "{} {}",
            if *op == '-' {
                "ବିଯୁକ୍ତ"
            } else {
                "ଯୁକ୍ତ"
            },
            render(x)?
        ),
        Expr::Group(x) => format!("ବାମ ବନ୍ଧନୀ {} ଡାହାଣ ବନ୍ଧନୀ", render(x)?),
        Expr::Power(x, p) => format!("{} ର ଘାତ {}", render(x)?, render(p)?),
        Expr::Binary(op, a, b) => {
            let left = render(a)?;
            let right = render(b)?;
            match op {
                '+' => format!("{} ଯୁକ୍ତ {}", left, right),
                '-' => format!("{} ବିଯୁକ୍ତ {}", left, right),
                '*' if matches!(a.as_ref(), Expr::Number(_))
                    && let Some(unit) = unit_reading(b) =>
                {
                    format!("{left} {unit}")
                }
                '*' => format!("{} ଗୁଣନ {}", left, right),
                '/' => format!("ଲବ {} ହର {}", left, right),
                _ => return None,
            }
        }
    })
}

/// Parse a complete, bounded formula and return its Odia reading.
pub(super) fn read(source: &str) -> Option<(SegmentKind, String, &'static str)> {
    let ascii = numerals::ascii(source);
    let normalized = latex(&ascii)?;
    let mut nesting = 0usize;
    let mut unary_run = 0usize;
    for c in normalized.chars() {
        if matches!(c, '(' | '{') {
            nesting += 1;
            if nesting > 64 {
                return None;
            }
        } else if matches!(c, ')' | '}') {
            nesting = nesting.saturating_sub(1);
        }
        if matches!(c, '+' | '-') {
            unary_run += 1;
            if unary_run > 64 {
                return None;
            }
        } else {
            unary_run = 0;
        }
    }
    // Avoid treating an isolated variable or number as a formula. Require an operator,
    // power, fraction, or equation; callers additionally constrain plain-text spans.
    let explicit = source.trim().starts_with(['$', '\\']);
    if !explicit && !normalized.contains(['+', '-', '*', '/', '=', '^', '²', '³']) {
        return None;
    }
    let algebraic = normalized.chars().any(|c| c.is_ascii_alphabetic())
        || normalized.contains(['^', '²', '³'])
        || (normalized.matches('/').count() == 1
            && normalized.chars().all(|c| c.is_ascii_digit() || c == '/'));
    // A bare numeric slash form can be a date; only accept numeric-only fractions
    // when they were explicitly marked as math.
    if !explicit && !algebraic {
        return None;
    }
    let words: Vec<_> = source.split_whitespace().collect();
    if words.len() == 2
        && words[0]
            .trim_start_matches(['+', '-'])
            .bytes()
            .all(|b| b.is_ascii_digit())
        && crate::vocabulary::unit(words[1]).is_some()
    {
        return None;
    }
    let parse = |s: &str| {
        let mut p = Parser {
            chars: s.chars().collect(),
            at: 0,
        };
        let e = p.expression()?;
        if p.at != p.chars.len() {
            return None;
        }
        render(&e)
    };
    let mut readings = Vec::new();
    for line in normalized.split(';').filter(|line| !line.is_empty()) {
        let mut parts = line.split('=');
        let first = parts.next()?;
        let leading_equality = first.is_empty();
        let mut spoken = if first.is_empty() {
            "ସମାନ".to_owned()
        } else {
            parse(first)?
        };
        for (index, part) in parts.enumerate() {
            if part.is_empty() {
                return None;
            }
            spoken.push_str(if leading_equality && index == 0 {
                " "
            } else {
                " ସମାନ "
            });
            spoken.push_str(&parse(part)?);
        }
        readings.push(spoken);
    }
    Some((
        SegmentKind::Formula,
        readings.join(" । "),
        "formula.elementary",
    ))
}
