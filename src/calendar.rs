use std::collections::HashSet;

use chrono::{naive::NaiveDate, Datelike, Duration, Weekday};
use serde::Deserialize;

const WORKWEEK: &[Weekday] = &[
    Weekday::Mon,
    Weekday::Tue,
    Weekday::Wed,
    Weekday::Thu,
    Weekday::Fri,
];

#[derive(Debug, PartialEq, Deserialize)]
pub struct Calendar {
    #[serde(default = "workweek")]
    working_days: HashSet<Weekday>,
    holidays: HashSet<NaiveDate>,
}

impl Calendar {
    pub fn workweek() -> Calendar {
        Self {
            working_days: workweek(),
            holidays: HashSet::new(),
        }
    }

    pub fn with_holidays(holidays: &[NaiveDate]) -> Calendar {
        let holidays: HashSet<_> = holidays.iter().cloned().collect();

        Self {
            working_days: workweek(),
            holidays,
        }
    }

    pub fn is_business_day(&self, date: NaiveDate) -> bool {
        let is_working_day = self.working_days.contains(&date.weekday());
        let is_holiday = self.holidays.contains(&date);
        is_working_day && !is_holiday
    }

    pub fn roll_forward(&self, date: NaiveDate) -> NaiveDate {
        let mut result = date;
        while !self.is_business_day(result) {
            result += Duration::days(1);
        }
        result
    }

    pub fn roll_backward(&self, date: NaiveDate) -> NaiveDate {
        let mut result = date;
        while !self.is_business_day(result) {
            result -= Duration::days(1);
        }
        result
    }

    pub fn next_business_day(&self, date: NaiveDate) -> NaiveDate {
        let mut result = date;
        loop {
            result += Duration::days(1);
            if self.is_business_day(result) {
                break;
            }
        }
        result
    }

    pub fn previous_business_day(&self, date: NaiveDate) -> NaiveDate {
        let mut result = date;
        loop {
            result -= Duration::days(1);
            if self.is_business_day(result) {
                break;
            }
        }
        result
    }

    pub fn add_business_days(&self, date: NaiveDate, delta: u32) -> NaiveDate {
        let mut result = self.roll_forward(date);
        for _ in 0..delta {
            result = self.next_business_day(result);
        }
        result
    }

    pub fn subtract_business_days(&self, date: NaiveDate, delta: u32) -> NaiveDate {
        let mut result = self.roll_forward(date);
        for _ in 0..delta {
            result = self.previous_business_day(result);
        }
        result
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
    fn sun_sub_2_business_is_wed() {
        let sun = NaiveDate::from_ymd(2022, 10, 02);
        let holiday_fri = NaiveDate::from_ymd(2022, 09, 30);
        let cal = Calendar::with_holidays(&[holiday_fri]);

        let business_wed = NaiveDate::from_ymd(2022, 09, 28);

        assert_eq!(cal.subtract_business_days(sun, 2), business_wed);
    }

    #[test]
    fn web_sub_2_business_is_mon() {
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
