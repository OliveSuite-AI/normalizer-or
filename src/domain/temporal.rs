//! Validated Gregorian dates and digital clocks.
use crate::{
    morphology::{Spoken, TailInflection, Word},
    numerals,
};

#[derive(Clone, Copy, Debug)]
pub(crate) enum DateFormat {
    Dotted,
    Iso,
    SlashDmy,
    SlashYmd,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct Date {
    day: u8,
    month: u8,
    year: u16,
    format: DateFormat,
}

impl Date {
    pub(crate) fn parse(text: &str) -> Option<Self> {
        let format = if text.contains('.') {
            DateFormat::Dotted
        } else if text.contains('/') {
            if text.split('/').next()?.len() == 4 {
                DateFormat::SlashYmd
            } else {
                DateFormat::SlashDmy
            }
        } else {
            DateFormat::Iso
        };
        let separator = match format {
            DateFormat::Dotted => '.',
            DateFormat::Iso => '-',
            DateFormat::SlashDmy | DateFormat::SlashYmd => '/',
        };
        let mut parts = text.split(separator);
        let (Some(first), Some(second), Some(third), None) =
            (parts.next(), parts.next(), parts.next(), parts.next())
        else {
            return None;
        };
        let (day, month, year) = if matches!(format, DateFormat::Iso | DateFormat::SlashYmd) {
            (third, second, first)
        } else {
            (first, second, third)
        };
        if year.len() != 4
            || !(1..=2).contains(&day.len())
            || !(1..=2).contains(&month.len())
            || (!matches!(format, DateFormat::Dotted) && (day.len() != 2 || month.len() != 2))
            || ![day, month, year]
                .iter()
                .all(|s| s.bytes().all(|b| b.is_ascii_digit()))
        {
            return None;
        }
        let day: u8 = day.parse().ok()?;
        let month: u8 = month.parse().ok()?;
        let year: u16 = year.parse().ok()?;
        let leap =
            year.is_multiple_of(4) && (!year.is_multiple_of(100) || year.is_multiple_of(400));
        let days = match month {
            2 if leap => 29,
            2 => 28,
            4 | 6 | 9 | 11 => 30,
            1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
            _ => return None,
        };
        if year == 0 || day == 0 || day > days {
            return None;
        }
        Some(Self {
            day,
            month,
            year,
            format,
        })
    }
    pub(crate) fn day(self) -> u8 {
        self.day
    }
    pub(crate) fn month(self) -> u8 {
        self.month
    }
    pub(crate) fn year(self) -> u16 {
        self.year
    }
    pub(crate) fn slash(self) -> bool {
        matches!(self.format, DateFormat::SlashDmy)
    }
    pub(crate) fn year_first_slash(self) -> bool {
        matches!(self.format, DateFormat::SlashYmd)
    }
    pub(crate) fn source_tail(self) -> Word {
        numerals::cardinal_tail(u64::from(
            if matches!(self.format, DateFormat::Iso | DateFormat::SlashYmd) {
                u16::from(self.day)
            } else {
                self.year
            },
        ))
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct DateTail(Option<TailInflection>);

impl DateTail {
    pub(crate) fn parse(date: Date, suffix: Option<&str>) -> Option<Self> {
        Some(Self(match suffix {
            Some(suffix) => Some(TailInflection::date(date.source_tail(), suffix)?),
            None => None,
        }))
    }
    pub(crate) fn is_locative(self) -> bool {
        self.0.is_some_and(TailInflection::is_locative)
    }
    pub(crate) fn apply(self, spoken: &mut Spoken) {
        if let Some(tail) = self.0 {
            tail.apply(spoken);
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct Clock {
    hour: u8,
    minute: u8,
}

impl Clock {
    pub(crate) fn parse(text: &str) -> Option<Self> {
        let (hour, minute) = text.split_once([':', '.'])?;
        if !(1..=2).contains(&hour.len())
            || minute.len() != 2
            || ![hour, minute]
                .iter()
                .all(|s| s.bytes().all(|b| b.is_ascii_digit()))
        {
            return None;
        }
        let hour = hour.parse().ok()?;
        let minute = minute.parse().ok()?;
        if hour > 23 || minute > 59 {
            return None;
        }
        Some(Self { hour, minute })
    }
    pub(crate) fn hour(self) -> u8 {
        self.hour
    }
    pub(crate) fn minute(self) -> u8 {
        self.minute
    }
    pub(crate) fn tail(self) -> Word {
        numerals::cardinal_tail(u64::from(if self.minute == 0 {
            self.hour
        } else {
            self.minute
        }))
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct ClockRange {
    start: Clock,
    end: Clock,
    ile: bool,
}

impl ClockRange {
    pub(crate) fn parse(source: &str, ile: bool) -> Option<Self> {
        let (start, end) = source.split_once(['-', '–'])?;
        if ![start, end].iter().all(|part| part.contains(['.', ':'])) {
            return None;
        }
        Some(Self {
            start: Clock::parse(start.trim())?,
            end: Clock::parse(end.trim())?,
            ile,
        })
    }
    pub(crate) fn start(self) -> Clock {
        self.start
    }
    pub(crate) fn end(self) -> Clock {
        self.end
    }
    pub(crate) fn connector(self) -> &'static str {
        if self.ile { "ile" } else { "ila" }
    }
}

#[cfg(test)]
mod notation_tests;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::numerals::Amount;

    #[test]
    fn gregorian_boundaries_and_iso_shape_are_validated_before_rendering() {
        for year in 1..=9999_u16 {
            let expected_leap =
                year.is_multiple_of(4) && (!year.is_multiple_of(100) || year.is_multiple_of(400));
            assert_eq!(
                Date::parse(&format!("29.02.{year:04}")).is_some(),
                expected_leap
            );
        }
        for input in [
            "31.04.2026",
            "00.01.2026",
            "01.00.2026",
            "2026-3-04",
            "2026-03-4",
        ] {
            assert!(Date::parse(input).is_none());
        }
    }

    #[test]
    fn clock_and_amount_private_types_cannot_hold_invalid_values() {
        assert!(Clock::parse("23:59").is_some());
        assert!(Clock::parse("00:00").is_some());
        for input in ["24:00", "12:60", "12:5", "1:2:3"] {
            assert!(Clock::parse(input).is_none());
        }
        assert_eq!(Amount::parse("-0,05").unwrap().minor(), 5);
        assert!(Amount::parse("1,005").is_none());
    }
}
