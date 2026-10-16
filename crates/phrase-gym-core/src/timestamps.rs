use chrono::{DateTime, ParseError, SecondsFormat, Timelike, Utc};

// Matches Python's datetime.isoformat() for UTC so stored text stays uniform with existing rows.
pub fn isoformat(moment: DateTime<Utc>) -> String {
    if moment.nanosecond() == 0 {
        moment.to_rfc3339_opts(SecondsFormat::Secs, false)
    } else {
        moment.to_rfc3339_opts(SecondsFormat::Micros, false)
    }
}

pub fn parse(text: &str) -> Result<DateTime<Utc>, ParseError> {
    DateTime::parse_from_rfc3339(text).map(|moment| moment.with_timezone(&Utc))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_like_existing_records_and_parses_offsets() {
        let whole = parse("2026-10-12T12:00:00+00:00").unwrap();
        assert_eq!(isoformat(whole), "2026-10-12T12:00:00+00:00");
        let fractional = parse("2026-10-12T14:00:00.120000+02:00").unwrap();
        assert_eq!(isoformat(fractional), "2026-10-12T12:00:00.120000+00:00");
        assert!(parse("2026-10-12 12:00").is_err());
    }
}
