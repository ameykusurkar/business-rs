use std::str::FromStr;

use chrono::naive::NaiveDate;

#[derive(Debug, PartialEq)]
struct FlexibleFormatDate(NaiveDate);

impl FromStr for FlexibleFormatDate {
    // TODO: Determine correct type
    type Err = chrono::ParseError;

    fn from_str(s: &str) -> Result<Self, <Self as FromStr>::Err> {
        // TODO: Handle ordinal suffixes like "1st"
        let nd = NaiveDate::parse_from_str(s, "%B %erd, %Y")?;
        Ok(Self(nd))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_date() {
        assert_eq!(
            "October 3rd, 2022".parse(),
            Ok(FlexibleFormatDate(NaiveDate::from_ymd(2022, 10, 3)))
        );
    }
}
