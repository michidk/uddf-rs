//! UDDF date-times, which real files write with and without a UTC offset, and sometimes
//! without a time of day.

use core::fmt;
use core::str::FromStr;

use jiff::civil::{Date, Time};

/// A UDDF date-time as written: a calendar date, an optional time of day and an optional
/// UTC offset, each kept exactly as the file states it.
///
/// UDDF types these elements `xs:dateTime`, whose offset is optional, and most writers
/// omit it: such a value is a local wall-clock time whose instant is unknown. This type
/// never assumes an offset. A `Z` is kept as [`Offset::Utc`], distinct from `+00:00`,
/// because at least one writer (Shearwater Cloud Desktop) writes `Z` on local times.
///
/// Accepted spellings, besides the full `2026-04-19T08:12:00.25+02:00`:
///
/// - a date alone, `2002-06-18`, and the truncated `2002-06-18T` (read the same, without a
///   time of day; the writer emits the first);
/// - a date with an offset and no time, `2002-06-18+02:00`;
/// - a time without seconds, `08:12`, or with a space instead of `T`;
/// - an offset as `+HH:MM`, `+HHMM` or `+HH`.
///
/// [`Display`](fmt::Display) writes the canonical `xs:dateTime` spelling, which a date
/// without a time of day cannot have: it is written as a bare date.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DateTime {
    /// The calendar date.
    pub date: Date,
    /// The time of day, absent where the file gives only a date.
    pub time: Option<Time>,
    /// The UTC offset, absent for a local time.
    pub offset: Option<Offset>,
}

/// The UTC offset of a [`DateTime`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Offset {
    /// Written as `Z`.
    Utc,
    /// Written as `+HH:MM` or `-HH:MM` (including `+00:00`).
    Fixed(jiff::tz::Offset),
}

impl Offset {
    /// The offset as a `jiff` offset; `Z` is zero.
    #[must_use]
    pub fn to_jiff(self) -> jiff::tz::Offset {
        match self {
            Self::Utc => jiff::tz::Offset::UTC,
            Self::Fixed(offset) => offset,
        }
    }
}

impl DateTime {
    /// A local date-time with no offset.
    #[must_use]
    pub fn local(value: jiff::civil::DateTime) -> Self {
        Self {
            date: value.date(),
            time: Some(value.time()),
            offset: None,
        }
    }

    /// The date and time of day, if the time of day is known.
    #[must_use]
    pub fn civil(&self) -> Option<jiff::civil::DateTime> {
        self.time.map(|time| self.date.to_datetime(time))
    }

    /// The instant, if both the time of day and the offset are known.
    #[must_use]
    pub fn timestamp(&self) -> Option<jiff::Timestamp> {
        self.offset?.to_jiff().to_timestamp(self.civil()?).ok()
    }
}

/// The text is not a date-time UDDF readers accept; see [`DateTime`] for the spellings.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ParseDateTimeError;

impl fmt::Display for ParseDateTimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("not a UDDF date-time")
    }
}

impl core::error::Error for ParseDateTimeError {}

/// Exactly `len` ASCII digits as a number.
fn digits<T: FromStr>(text: &str, len: usize) -> Option<T> {
    (text.len() == len && text.bytes().all(|b| b.is_ascii_digit())).then(|| text.parse().ok())?
}

fn parse_date(text: &str) -> Option<Date> {
    let mut parts = text.split('-');
    let (year, month, day) = (parts.next()?, parts.next()?, parts.next()?);
    if parts.next().is_some() {
        return None;
    }
    Date::new(digits(year, 4)?, digits(month, 2)?, digits(day, 2)?).ok()
}

fn parse_time(text: &str) -> Option<Time> {
    let (clock, fraction) = text.split_once('.').unwrap_or((text, ""));
    let mut parts = clock.split(':');
    let (hour, minute) = (parts.next()?, parts.next()?);
    let second = parts.next().unwrap_or("00");
    if parts.next().is_some() || (fraction.is_empty() && text.contains('.')) || fraction.len() > 9 {
        return None;
    }
    let nanos = if fraction.is_empty() {
        0
    } else {
        digits::<i32>(&format!("{fraction:0<9}"), 9)?
    };
    Time::new(
        digits(hour, 2)?,
        digits(minute, 2)?,
        digits(second, 2)?,
        nanos,
    )
    .ok()
}

/// `+HH:MM`, `+HHMM` or `+HH`, and the same with `-`.
fn parse_offset(zone: &str) -> Option<jiff::tz::Offset> {
    let (negative, rest) = match zone.split_at_checked(1)? {
        ("+", rest) => (false, rest),
        ("-", rest) => (true, rest),
        _ => return None,
    };
    let rest = rest.replacen(':', "", 1);
    let (hours, minutes) = match rest.len() {
        2 => (rest.as_str(), "00"),
        4 => rest.split_at(2),
        _ => return None,
    };
    let (hours, minutes): (i32, i32) = (digits(hours, 2)?, digits(minutes, 2)?);
    if hours > 23 || minutes > 59 {
        return None;
    }
    let seconds = hours
        .checked_mul(3600)?
        .checked_add(minutes.checked_mul(60)?)?;
    jiff::tz::Offset::from_seconds(if negative {
        seconds.checked_neg()?
    } else {
        seconds
    })
    .ok()
}

impl FromStr for DateTime {
    type Err = ParseDateTimeError;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let text = text.trim();
        let (date, rest) = text.split_at_checked(10).ok_or(ParseDateTimeError)?;
        let date = parse_date(date).ok_or(ParseDateTimeError)?;
        let rest = rest.strip_prefix(['T', 't', ' ']).unwrap_or(rest);
        let zone_at = rest.find(['Z', 'z', '+', '-']).unwrap_or(rest.len());
        let (time, zone) = rest.split_at(zone_at);
        let time = match time {
            "" => None,
            time => Some(parse_time(time).ok_or(ParseDateTimeError)?),
        };
        let offset = match zone {
            "" => None,
            "Z" | "z" => Some(Offset::Utc),
            zone => Some(Offset::Fixed(parse_offset(zone).ok_or(ParseDateTimeError)?)),
        };
        Ok(Self { date, time, offset })
    }
}

impl fmt::Display for DateTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let date = self.date;
        write!(
            f,
            "{:04}-{:02}-{:02}",
            date.year(),
            date.month(),
            date.day()
        )?;
        if let Some(time) = self.time {
            write!(
                f,
                "T{:02}:{:02}:{:02}",
                time.hour(),
                time.minute(),
                time.second()
            )?;
            let nanos = time.subsec_nanosecond();
            if nanos != 0 {
                let fraction = format!("{nanos:09}");
                write!(f, ".{}", fraction.trim_end_matches('0'))?;
            }
        }
        match self.offset {
            None => Ok(()),
            Some(Offset::Utc) => f.write_str("Z"),
            Some(Offset::Fixed(offset)) => {
                let seconds = offset.seconds();
                let sign = if seconds < 0 { '-' } else { '+' };
                let minutes = seconds.unsigned_abs() / 60;
                write!(f, "{sign}{:02}:{:02}", minutes / 60, minutes % 60)
            }
        }
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for DateTime {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(self)
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for DateTime {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let text = <std::borrow::Cow<'de, str>>::deserialize(deserializer)?;
        text.parse().map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn round(text: &str) -> String {
        text.parse::<DateTime>().unwrap().to_string()
    }

    #[test]
    fn keeps_every_spelling_that_matters() {
        assert_eq!(
            round("2026-04-19T08:12:00+02:00"),
            "2026-04-19T08:12:00+02:00"
        );
        assert_eq!(round("2026-09-08T15:18:10Z"), "2026-09-08T15:18:10Z");
        assert_eq!(
            round("2026-09-08T15:18:10+00:00"),
            "2026-09-08T15:18:10+00:00"
        );
        assert_eq!(round("2026-04-17T11:49:23"), "2026-04-17T11:49:23");
        assert_eq!(
            round("2026-09-02T18:27:26.302137+00:00"),
            "2026-09-02T18:27:26.302137+00:00"
        );
        assert_eq!(
            round("2026-07-03T09:14:00+12:45"),
            "2026-07-03T09:14:00+12:45"
        );
        assert_eq!(
            round("1990-01-01T10:00:00-0330"),
            "1990-01-01T10:00:00-03:30"
        );
        assert_eq!(
            round("1990-01-01T10:00:00.5-03"),
            "1990-01-01T10:00:00.5-03:00"
        );
    }

    #[test]
    fn reads_truncated_writer_output() {
        let date_only: DateTime = "2002-06-18".parse().unwrap();
        assert_eq!(date_only.time, None);
        assert_eq!(date_only.offset, None);
        assert_eq!("2002-06-18T".parse::<DateTime>().unwrap(), date_only);
        assert_eq!(round("2002-06-18+02:00"), "2002-06-18+02:00");
        assert_eq!(round("2002-06-18 07:05"), "2002-06-18T07:05:00");
    }

    #[test]
    fn rejects_what_is_not_a_date_time() {
        for bad in [
            "",
            "yesterday",
            "2002-13-01",
            "2002-02-30",
            "2002-06-18T25:00:00",
            "2002-06-18T10:00:00+5",
            "2002-06-18T10:00:00.",
            "2002",
        ] {
            assert!(bad.parse::<DateTime>().is_err(), "{bad}");
        }
    }

    #[test]
    fn exposes_the_instant_only_when_it_is_known() {
        let local: DateTime = "2026-04-17T11:49:23".parse().unwrap();
        assert!(local.timestamp().is_none());
        assert!(local.civil().is_some());
        let zoned: DateTime = "2026-04-19T08:12:00+02:00".parse().unwrap();
        assert_eq!(zoned.timestamp().unwrap().as_second(), 1_776_579_120);
    }
}
