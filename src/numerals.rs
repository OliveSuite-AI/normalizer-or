//! Exact decimal parsing and Indian-scale Odia rendering. No floating point.
use crate::vocabulary::SMALL;

pub(crate) const MAGNITUDE_LIMIT: u64 = 1_000_000_000_000_000_000;

pub(crate) fn digit(ch: char) -> Option<u8> {
    match ch {
        '0'..='9' => Some(ch as u8 - b'0'),
        '୦'..='୯' => Some((ch as u32 - '୦' as u32) as u8),
        _ => None,
    }
}

pub(crate) fn ascii(text: &str) -> String {
    text.chars()
        .map(|ch| digit(ch).map_or(ch, |d| char::from(b'0' + d)))
        .collect()
}

pub(crate) fn cardinal(value: u64) -> String {
    if value < 100 {
        return SMALL[value as usize].to_owned();
    }
    for (scale, name) in [
        (10_000_000, "କୋଟି"),
        (100_000, "ଲକ୍ଷ"),
        (1000, "ହଜାର"),
        (100, "ଶହ"),
    ] {
        if value >= scale {
            let mut text = format!("{} {}", cardinal(value / scale), name);
            if !value.is_multiple_of(scale) {
                text.push(' ');
                text.push_str(&cardinal(value % scale));
            }
            return text;
        }
    }
    unreachable!("all nonnegative integers have a scale")
}

pub(crate) fn digits(text: &str) -> String {
    text.chars()
        .filter_map(digit)
        .map(|d| SMALL[d as usize])
        .collect::<Vec<_>>()
        .join(" ")
}

#[derive(Debug)]
pub(crate) struct Number<'a> {
    pub(crate) integer: u64,
    pub(crate) fraction: &'a str,
    pub(crate) sign: &'static str,
    pub(crate) padded: bool,
    whole: &'a str,
}

impl<'a> Number<'a> {
    // Input has already been converted with the shared ASCII/Odia digit helper.
    pub(crate) fn parse(text: &'a str, allow_padding: bool) -> Option<Self> {
        let (sign, body) = if let Some(body) = text.strip_prefix('-') {
            ("ବିଯୁକ୍ତ ", body)
        } else if let Some(body) = text.strip_prefix('+') {
            ("ଯୁକ୍ତ ", body)
        } else {
            ("", text)
        };
        let mut parts = body.split('.');
        let whole = parts.next()?;
        let fraction = parts.next().unwrap_or("");
        if parts.next().is_some()
            || (body.contains('.') && (fraction.is_empty() || fraction.len() > 9))
            || !fraction.bytes().all(|b| b.is_ascii_digit())
        {
            return None;
        }
        let groups: Vec<_> = whole.split(',').collect();
        if groups
            .iter()
            .any(|g| g.is_empty() || !g.bytes().all(|b| b.is_ascii_digit()))
        {
            return None;
        }
        if groups.len() > 1 {
            let western =
                (1..=3).contains(&groups[0].len()) && groups[1..].iter().all(|g| g.len() == 3);
            let indian = (1..=2).contains(&groups[0].len())
                && groups.last()?.len() == 3
                && groups[1..groups.len() - 1].iter().all(|g| g.len() == 2);
            if !(western || indian) || groups[0].starts_with('0') {
                return None;
            }
        }
        let padded = groups.len() == 1 && whole.len() > 1 && whole.starts_with('0');
        if padded && (!allow_padding || !fraction.is_empty()) {
            return None;
        }
        let mut integer = 0u64;
        for byte in whole.bytes().filter(|b| *b != b',') {
            integer = integer
                .checked_mul(10)?
                .checked_add(u64::from(byte - b'0'))?;
            if integer >= MAGNITUDE_LIMIT {
                return None;
            }
        }
        Some(Self {
            integer,
            fraction,
            sign,
            padded,
            whole,
        })
    }

    pub(crate) fn render(&self, cardinal_padding: bool) -> String {
        let mut text = format!(
            "{}{}",
            self.sign,
            if self.padded && !cardinal_padding {
                digits(self.whole)
            } else {
                cardinal(self.integer)
            }
        );
        if !self.fraction.is_empty() {
            text.push_str(" ଦଶମିକ ");
            text.push_str(&digits(self.fraction));
        }
        text
    }

    pub(crate) fn money(&self) -> Option<String> {
        if self.fraction.len() > 2 || self.padded {
            return None;
        }
        let mut text = format!("{}{} ଟଙ୍କା", self.sign, cardinal(self.integer));
        let minor: u64 = match self.fraction.len() {
            0 => 0,
            1 => self.fraction.parse::<u64>().ok()? * 10,
            _ => self.fraction.parse().ok()?,
        };
        if minor != 0 {
            text.push_str(&format!(" {} ପଇସା", cardinal(minor)));
        }
        Some(text)
    }
}
