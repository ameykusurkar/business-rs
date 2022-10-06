#![deny(missing_docs)]

//! A crate for doing business day calculations. It is a Rust implementation of the Ruby
//! [business](https://github.com/gocardless/business) gem.
//!
//! Let's dive right in with an example. For more details, see
//! [`Calendar`](https://docs.rs/business/latest/business/struct.Calendar.html).
//!
//! ```
//! use chrono::NaiveDate;
//!
//! let xmas = NaiveDate::from_ymd(2020, 12, 25); // Friday
//!
//! let cal = business::Calendar::with_holidays(&[xmas]);
//!
//! assert_eq!(cal.is_business_day(xmas), false);
//!
//! // The earliest business day
//! assert_eq!(cal.roll_forward(xmas), NaiveDate::from_ymd(2020, 12, 28));
//!
//! let xmas_eve = NaiveDate::from_ymd(2020, 12, 24);
//! assert_eq!(cal.is_business_day(xmas_eve), true);
//!
//! // Skips over weekend and business holidays
//! assert_eq!(cal.add_business_days(xmas_eve, 2), NaiveDate::from_ymd(2020, 12, 29));
//! ```
//!
//! # Building a `Calendar` from YAML
//!
//! The YAML has to be in the following format:
//! ```yaml
//! # Defaults to Mon-Fri if omitted
//! working_days:
//!   - monday
//!   - tuesday
//!   - wednesday
//!   - thursday
//!   - friday
//! # ISO 8601 dates, defaults to no holidays if omitted
//! holidays:
//!   - 2017-12-25
//!   - 2017-12-26
//! ```
//! A calendar can be built as such:
//! ```
//! # use business::Calendar;
//! let yml = std::fs::read_to_string("examples/basic/cal.yml").unwrap();
//! let cal: Calendar = serde_yaml::from_str(&yml).unwrap();
//! ```

use std::collections::HashSet;
use std::ops::Add;

use chrono::{naive::NaiveDate, Datelike, Duration, Weekday};
use serde::Deserialize;

const WORKWEEK: &[Weekday] = &[
    Weekday::Mon,
    Weekday::Tue,
    Weekday::Wed,
    Weekday::Thu,
    Weekday::Fri,
];

/// A calendar for doing business date calculations.
///
/// ```
/// use chrono::NaiveDate;
///
/// let xmas = NaiveDate::from_ymd(2020, 12, 25); // Friday
///
/// let cal = business::Calendar::with_holidays(&[xmas]);
///
/// assert_eq!(cal.is_business_day(xmas), false);
///
/// // The earliest business day
/// assert_eq!(cal.roll_forward(xmas), NaiveDate::from_ymd(2020, 12, 28));
///
/// let xmas_eve = NaiveDate::from_ymd(2020, 12, 24);
/// assert_eq!(cal.is_business_day(xmas_eve), true);
///
/// // Skips over weekend and business holidays
/// assert_eq!(cal.add_business_days(xmas_eve, 2), NaiveDate::from_ymd(2020, 12, 29));
/// ```
#[derive(Debug, PartialEq, Deserialize)]
pub struct Calendar {
    /// Working days of the week
    #[serde(default = "workweek")]
    pub working_days: HashSet<Weekday>,
    /// Holiday dates, regardless of the day of the week
    pub holidays: HashSet<NaiveDate>,
}

impl Calendar {
    /// Creates a `Calendar` with Mon-Fri as working days and no holidays.
    pub fn workweek() -> Calendar {
        Self {
            working_days: workweek(),
            holidays: HashSet::new(),
        }
    }

    /// Creates a `Calendar` with Mon-Fri as working days and the specified holidays.
    pub fn with_holidays(holidays: &[NaiveDate]) -> Calendar {
        let holidays: HashSet<_> = holidays.iter().cloned().collect();

        Self {
            working_days: workweek(),
            holidays,
        }
    }

    /// Returns `true` if the date is a working day and not a holiday.
    ///
    /// # Examples
    ///
    /// ```
    /// # use chrono::NaiveDate;
    /// # use business::Calendar;
    /// let cal = Calendar::with_holidays(&[NaiveDate::from_ymd(2020, 12, 25)]);
    /// assert_eq!(cal.is_business_day(NaiveDate::from_ymd(2020, 12, 25)), false);
    /// assert_eq!(cal.is_business_day(NaiveDate::from_ymd(2020, 12, 24)), true);
    ///
    /// // Saturday
    /// assert_eq!(cal.is_business_day(NaiveDate::from_ymd(2020, 12, 26)), false);
    /// ```
    pub fn is_business_day<D: IntoDate>(&self, date: D) -> bool {
        let is_working_day = self.working_days.contains(&date.weekday());
        let is_holiday = self.holidays.contains(&date.into_date());
        is_working_day && !is_holiday
    }

    /// Rolls forward to the next business day. If the date is already a business day,
    /// the same date will be returned.
    ///
    /// # Examples
    ///
    /// ```
    /// # use chrono::NaiveDate;
    /// # use business::Calendar;
    /// let cal = Calendar::workweek();
    /// let sat = NaiveDate::from_ymd(2022, 10, 1);
    /// let mon = NaiveDate::from_ymd(2022, 10, 3);
    /// assert_eq!(cal.roll_forward(sat), mon);
    /// assert_eq!(cal.roll_forward(mon), mon);
    /// ```
    pub fn roll_forward<D>(&self, date: D) -> D
    where
        D: IntoDate + Add<Duration, Output = D>,
    {
        let mut result = date;
        while !self.is_business_day(result) {
            result = result + Duration::days(1);
        }
        result
    }

    /// Rolls backward to the previous business day. If the date is already a business day,
    /// the same date will be returned.
    ///
    /// # Examples
    ///
    /// ```
    /// # use chrono::NaiveDate;
    /// # use business::Calendar;
    /// let cal = Calendar::workweek();
    /// let fri = NaiveDate::from_ymd(2022, 9, 30);
    /// let sun = NaiveDate::from_ymd(2022, 10, 2);
    /// assert_eq!(cal.roll_backward(sun), fri);
    /// assert_eq!(cal.roll_backward(fri), fri);
    /// ```
    pub fn roll_backward<D>(&self, date: D) -> D
    where
        D: IntoDate + Add<Duration, Output = D>,
    {
        let mut result = date;
        while !self.is_business_day(result) {
            result = result + Duration::days(-1);
        }
        result
    }

    /// Rolls forward to the next business day regardless of whether the given
    /// date is already a business day.
    ///
    /// # Examples
    ///
    /// ```
    /// # use chrono::NaiveDate;
    /// # use business::Calendar;
    /// let cal = Calendar::workweek();
    /// let sat = NaiveDate::from_ymd(2022, 10, 1);
    /// let mon = NaiveDate::from_ymd(2022, 10, 3);
    /// let tue = NaiveDate::from_ymd(2022, 10, 4);
    /// assert_eq!(cal.next_business_day(sat), mon);
    /// assert_eq!(cal.next_business_day(mon), tue);
    /// ```
    pub fn next_business_day<D>(&self, date: D) -> D
    where
        D: IntoDate + Add<Duration, Output = D>,
    {
        let mut result = date;
        loop {
            result = result + Duration::days(1);
            if self.is_business_day(result) {
                break;
            }
        }
        result
    }

    /// Rolls backward to the previous business day regardless of whether the given
    /// date is already a business day.
    ///
    /// # Examples
    ///
    /// ```
    /// # use chrono::NaiveDate;
    /// # use business::Calendar;
    /// let cal = Calendar::workweek();
    /// let thu = NaiveDate::from_ymd(2022, 9, 29);
    /// let fri = NaiveDate::from_ymd(2022, 9, 30);
    /// let sun = NaiveDate::from_ymd(2022, 10, 2);
    /// assert_eq!(cal.previous_business_day(sun), fri);
    /// assert_eq!(cal.previous_business_day(fri), thu);
    /// ```
    pub fn previous_business_day<D>(&self, date: D) -> D
    where
        D: IntoDate + Add<Duration, Output = D>,
    {
        let mut result = date;
        loop {
            result = result + Duration::days(-1);
            if self.is_business_day(result) {
                break;
            }
        }
        result
    }

    /// Adds business days to the given date. If the date is not a business day, counting will
    /// start from the next business day.
    ///
    /// # Examples
    ///
    /// ```
    /// # use chrono::NaiveDate;
    /// # use business::Calendar;
    /// let cal = Calendar::workweek();
    /// let fri = NaiveDate::from_ymd(2022, 9, 30);
    /// let sun = NaiveDate::from_ymd(2022, 10, 2);
    /// let mon = NaiveDate::from_ymd(2022, 10, 3);
    /// let tue = NaiveDate::from_ymd(2022, 10, 4);
    /// assert_eq!(cal.add_business_days(mon, 1), tue);
    /// assert_eq!(cal.add_business_days(fri, 1), mon);
    /// assert_eq!(cal.add_business_days(sun, 1), tue);
    /// ```
    pub fn add_business_days<D>(&self, date: D, delta: u32) -> D
    where
        D: IntoDate + Add<Duration, Output = D>,
    {
        let mut result = self.roll_forward(date);
        for _ in 0..delta {
            result = self.next_business_day(result);
        }
        result
    }

    /// Subtracts business days from the given date. If the date is not a business day, counting
    /// will start from the previous business day.
    ///
    /// # Examples
    ///
    /// ```
    /// # use chrono::NaiveDate;
    /// # use business::Calendar;
    /// let cal = Calendar::workweek();
    /// let thu = NaiveDate::from_ymd(2022, 9, 29);
    /// let fri = NaiveDate::from_ymd(2022, 9, 30);
    /// let sun = NaiveDate::from_ymd(2022, 10, 2);
    /// let mon = NaiveDate::from_ymd(2022, 10, 3);
    /// assert_eq!(cal.subtract_business_days(fri, 1), thu);
    /// assert_eq!(cal.subtract_business_days(mon, 1), fri);
    /// assert_eq!(cal.subtract_business_days(sun, 1), thu);
    /// ```
    pub fn subtract_business_days<D>(&self, date: D, delta: u32) -> D
    where
        D: IntoDate + Add<Duration, Output = D>,
    {
        let mut result = self.roll_backward(date);
        for _ in 0..delta {
            result = self.previous_business_day(result);
        }
        result
    }
}

/// Types that can be converted into a [`NaiveDate`].
///
/// Since the type is [`Datelike`], there is already a default implementation for
/// `into_date`. Override the implementation if there is an efficient way to convert the type.
pub trait IntoDate: Datelike + Copy {
    /// Converts this type into [`NaiveDate`].
    fn into_date(self) -> NaiveDate {
        NaiveDate::from_ymd(self.year(), self.month(), self.day())
    }
}

impl IntoDate for NaiveDate {
    fn into_date(self) -> NaiveDate {
        self
    }
}

fn workweek() -> HashSet<Weekday> {
    WORKWEEK.iter().cloned().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sat_is_not_business() {
        let cal = Calendar::workweek();
        let saturday = NaiveDate::from_ymd(2022, 10, 01);

        assert_eq!(cal.is_business_day(saturday), false);
    }

    #[test]
    fn mon_is_business() {
        let cal = Calendar::workweek();
        let monday = NaiveDate::from_ymd(2022, 10, 03);

        assert_eq!(cal.is_business_day(monday), true);
    }

    #[test]
    fn mon_holiday_is_not_business() {
        let monday = NaiveDate::from_ymd(2022, 10, 03);
        let cal = Calendar::with_holidays(&[monday]);

        assert_eq!(cal.is_business_day(monday), false);
    }

    #[test]
    fn sat_rolls_forward_to_tues() {
        let sat = NaiveDate::from_ymd(2022, 10, 01);
        let holiday_mon = NaiveDate::from_ymd(2022, 10, 03);
        let cal = Calendar::with_holidays(&[holiday_mon]);

        let business_tue = NaiveDate::from_ymd(2022, 10, 04);

        assert_eq!(cal.roll_forward(sat), business_tue);
    }

    #[test]
    fn mon_rolls_forward_same_day() {
        let mon = NaiveDate::from_ymd(2022, 10, 03);
        let cal = Calendar::workweek();

        assert_eq!(cal.roll_forward(mon), mon);
    }

    #[test]
    fn sun_rolls_backward_to_thu() {
        let sun = NaiveDate::from_ymd(2022, 10, 02);
        let holiday_fri = NaiveDate::from_ymd(2022, 09, 30);
        let cal = Calendar::with_holidays(&[holiday_fri]);

        let business_thu = NaiveDate::from_ymd(2022, 09, 29);

        assert_eq!(cal.roll_backward(sun), business_thu);
    }

    #[test]
    fn mon_rolls_backward_same_day() {
        let mon = NaiveDate::from_ymd(2022, 10, 03);
        let cal = Calendar::workweek();

        assert_eq!(cal.roll_backward(mon), mon);
    }

    #[test]
    fn sat_next_business_is_tues() {
        let sat = NaiveDate::from_ymd(2022, 10, 01);
        let holiday_mon = NaiveDate::from_ymd(2022, 10, 03);
        let cal = Calendar::with_holidays(&[holiday_mon]);

        let business_tue = NaiveDate::from_ymd(2022, 10, 04);

        assert_eq!(cal.next_business_day(sat), business_tue);
    }

    #[test]
    fn mon_next_business_is_tues() {
        let mon = NaiveDate::from_ymd(2022, 10, 03);
        let cal = Calendar::with_holidays(&[]);

        let tue = NaiveDate::from_ymd(2022, 10, 04);
        assert_eq!(cal.next_business_day(mon), tue);
    }

    #[test]
    fn sun_previous_business_is_thu() {
        let sun = NaiveDate::from_ymd(2022, 10, 02);
        let holiday_fri = NaiveDate::from_ymd(2022, 09, 30);
        let cal = Calendar::with_holidays(&[holiday_fri]);

        let business_thu = NaiveDate::from_ymd(2022, 09, 29);

        assert_eq!(cal.previous_business_day(sun), business_thu);
    }

    #[test]
    fn mon_previous_business_is_fri() {
        let mon = NaiveDate::from_ymd(2022, 10, 03);
        let cal = Calendar::with_holidays(&[]);

        let fri = NaiveDate::from_ymd(2022, 09, 30);
        assert_eq!(cal.previous_business_day(mon), fri);
    }

    #[test]
    fn sat_add_2_business_is_thu() {
        let sat = NaiveDate::from_ymd(2022, 10, 01);
        let holiday_tues = NaiveDate::from_ymd(2022, 10, 04);
        let cal = Calendar::with_holidays(&[holiday_tues]);

        let business_thu = NaiveDate::from_ymd(2022, 10, 06);

        assert_eq!(cal.add_business_days(sat, 2), business_thu);
    }

    #[test]
    fn mon_add_2_business_is_wed() {
        let mon = NaiveDate::from_ymd(2022, 10, 03);
        let cal = Calendar::workweek();

        let wed = NaiveDate::from_ymd(2022, 10, 05);

        assert_eq!(cal.add_business_days(mon, 2), wed);
    }

    #[test]
    fn sun_sub_2_business_is_thu() {
        let sun = NaiveDate::from_ymd(2022, 10, 02);
        let holiday_fri = NaiveDate::from_ymd(2022, 09, 30);
        let cal = Calendar::with_holidays(&[holiday_fri]);

        let business_thu = NaiveDate::from_ymd(2022, 09, 27);

        assert_eq!(cal.subtract_business_days(sun, 2), business_thu);
    }

    #[test]
    fn wed_sub_2_business_is_mon() {
        let wed = NaiveDate::from_ymd(2022, 10, 05);
        let cal = Calendar::workweek();

        let mon = NaiveDate::from_ymd(2022, 10, 03);

        assert_eq!(cal.subtract_business_days(wed, 2), mon);
    }

    #[test]
    fn parse_yaml() {
        let input = "
            working_days:
                - monday
                - tuesday
                - friday

            holidays:
              - 2022-01-01
              - 2012-12-25
        ";
        let cal: Calendar = serde_yaml::from_str(input).unwrap();

        let expected = Calendar {
            working_days: HashSet::from([Weekday::Mon, Weekday::Tue, Weekday::Fri]),
            holidays: HashSet::from([
                NaiveDate::from_ymd(2022, 1, 1),
                NaiveDate::from_ymd(2012, 12, 25),
            ]),
        };

        assert_eq!(cal, expected);
    }

    #[test]
    fn parse_yaml_with_defaults() {
        let input = "
            holidays:
              - 2022-01-01
              - 2012-12-25
        ";
        let cal: Calendar = serde_yaml::from_str(input).unwrap();

        let expected = Calendar::with_holidays(&[
            NaiveDate::from_ymd(2022, 1, 1),
            NaiveDate::from_ymd(2012, 12, 25),
        ]);

        assert_eq!(cal, expected);
    }
}
