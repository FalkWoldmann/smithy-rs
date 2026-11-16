/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

// Some of the functions in this file are unused when disabling certain features
#![allow(dead_code)]

use jiff::civil::DateTime;
use jiff::SignedDuration;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Truncates the subseconds from the given `SystemTime` to zero.
pub(crate) fn truncate_subsecs(time: SystemTime) -> SystemTime {
    match time.duration_since(UNIX_EPOCH) {
        Ok(since_epoch) => UNIX_EPOCH + Duration::from_secs(since_epoch.as_secs()),
        // Times before the epoch need to be rounded away from it in order to
        // only ever go back in time, just like for times after the epoch.
        Err(err) => {
            let until_epoch = err.duration();
            let secs = until_epoch.as_secs() + u64::from(until_epoch.subsec_nanos() > 0);
            UNIX_EPOCH - Duration::from_secs(secs)
        }
    }
}

/// Converts a `SystemTime` to its calendar representation in UTC.
///
/// This uses a civil date-time rather than a `jiff::Timestamp` since the latter
/// doesn't cover the first and last day of the range of years supported here.
fn to_utc(time: SystemTime) -> DateTime {
    const UNIX_EPOCH_UTC: DateTime = DateTime::constant(1970, 1, 1, 0, 0, 0, 0);

    SignedDuration::system_until(UNIX_EPOCH, time)
        .and_then(|since_epoch| UNIX_EPOCH_UTC.checked_add(since_epoch))
        .expect("signing time must be between the years -9999 and 9999")
}

/// Formats a `SystemTime` in `YYYYMMDD` format.
pub(crate) fn format_date(time: SystemTime) -> String {
    let time = to_utc(time);
    format!("{:04}{:02}{:02}", time.year(), time.month(), time.day())
}

/// Formats a `SystemTime` in `YYYYMMDD'T'HHMMSS'Z'` format.
pub(crate) fn format_date_time(time: SystemTime) -> String {
    let time = to_utc(time);
    format!(
        "{:04}{:02}{:02}T{:02}{:02}{:02}Z",
        time.year(),
        time.month(),
        time.day(),
        time.hour(),
        time.minute(),
        time.second()
    )
}

/// Parse functions that are only needed for unit tests.
#[cfg(test)]
pub(crate) mod test_parsers {
    use jiff::civil;
    use jiff::tz::TimeZone;
    use std::{borrow::Cow, error::Error, fmt, time::SystemTime};

    fn to_system_time(date_time: civil::DateTime) -> Result<SystemTime, ParseError> {
        date_time
            .to_zoned_datetime(TimeZone::UTC)
            .and_then(|zoned| SystemTime::try_from(zoned.timestamp()))
            .map_err(|err| ParseError(err.to_string().into()))
    }

    /// Parses `YYYYMMDD'T'HHMMSS'Z'` formatted dates into a `SystemTime`.
    pub(crate) fn parse_date_time(date_time_str: &str) -> Result<SystemTime, ParseError> {
        civil::DateTime::strptime("%Y%m%dT%H%M%SZ", date_time_str)
            .map_err(|err| ParseError(err.to_string().into()))
            .and_then(to_system_time)
    }

    /// Parses `YYYYMMDD` formatted dates into a `SystemTime`.
    pub(crate) fn parse_date(date_str: &str) -> Result<SystemTime, ParseError> {
        civil::Date::strptime("%Y%m%d", date_str)
            .map_err(|err| ParseError(err.to_string().into()))
            .and_then(|date| to_system_time(date.at(0, 0, 0, 0)))
    }

    #[derive(Debug)]
    pub(crate) struct ParseError(Cow<'static, str>);

    impl fmt::Display for ParseError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "failed to parse time: {}", self.0)
        }
    }

    impl Error for ParseError {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::date_time::test_parsers::{parse_date, parse_date_time};

    fn system_time(rfc3339: &str) -> SystemTime {
        let timestamp: jiff::Timestamp = rfc3339.parse().unwrap();
        timestamp.try_into().unwrap()
    }

    // TODO(https://github.com/smithy-lang/smithy-rs/issues/1857)
    #[cfg(not(any(target_arch = "powerpc", target_arch = "x86", target_os = "windows")))]
    #[test]
    fn date_format() {
        let time = system_time("2039-02-04T23:01:09.104Z");
        assert_eq!("20390204", format_date(time));
        let time = system_time("0100-01-02T00:00:00.000Z");
        assert_eq!("01000102", format_date(time));
    }

    // TODO(https://github.com/smithy-lang/smithy-rs/issues/1857)
    #[cfg(not(any(target_arch = "powerpc", target_arch = "x86", target_os = "windows")))]
    #[test]
    fn date_time_format() {
        let time = system_time("2039-02-04T23:01:09.104Z");
        assert_eq!("20390204T230109Z", format_date_time(time));
        let time = system_time("0100-01-02T00:00:00.000Z");
        assert_eq!("01000102T000000Z", format_date_time(time));
    }

    #[test]
    fn formats_full_range_of_four_digit_years() {
        // 0001-01-01T00:00:00Z and 9999-12-31T23:59:59.999Z
        let first = UNIX_EPOCH - Duration::from_secs(62_135_596_800);
        let last = UNIX_EPOCH + Duration::new(253_402_300_799, 999_000_000);

        assert_eq!("00010101", format_date(first));
        assert_eq!("00010101T000000Z", format_date_time(first));
        assert_eq!("99991231", format_date(last));
        assert_eq!("99991231T235959Z", format_date_time(last));
    }

    #[test]
    fn date_time_roundtrip() {
        let time = parse_date_time("20150830T123600Z").unwrap();
        assert_eq!("20150830T123600Z", format_date_time(time));
    }

    #[test]
    fn date_roundtrip() {
        let time = parse_date("20150830").unwrap();
        assert_eq!("20150830", format_date(time));
    }

    // TODO(https://github.com/smithy-lang/smithy-rs/issues/1857)
    #[cfg(not(any(target_arch = "powerpc", target_arch = "x86")))]
    #[test]
    fn test_truncate_subsecs() {
        let time = system_time("2039-02-04T23:01:09.104Z");
        let expected = system_time("2039-02-04T23:01:09.000Z");
        assert_eq!(expected, truncate_subsecs(time));

        // Truncation always goes back in time, including for times before the epoch
        let time = system_time("1969-12-31T23:59:58.104Z");
        let expected = system_time("1969-12-31T23:59:58.000Z");
        assert_eq!(expected, truncate_subsecs(time));
        assert_eq!(expected, truncate_subsecs(expected));
    }
}
