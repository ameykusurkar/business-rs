use std::collections::HashSet;

use chrono::{naive::NaiveDate, Datelike, Duration, Weekday};
use serde::Deserialize;

const WORK_WEEK: [Weekday; 5] = [
    Weekday::Mon,
    Weekday::Tue,
    Weekday::Wed,
    Weekday::Thu,
    Weekday::Fri,
];

#[derive(Debug, PartialEq, Deserialize)]
#[serde(try_from = "CalendarUnchecked")]
pub struct Calendar {
    working_days: Vec<Weekday>,
    holidays: HashSet<NaiveDate>,
    extra_working_dates: HashSet<NaiveDate>,
}

#[derive(Debug, PartialEq, Deserialize)]
struct CalendarUnchecked {
    working_days: Option<Vec<Weekday>>,
    holidays: Option<Vec<NaiveDate>>,
    extra_working_dates: Option<Vec<NaiveDate>>,
}

impl TryFrom<CalendarUnchecked> for Calendar {
    type Error = Error;
    fn try_from(cal: CalendarUnchecked) -> Result<Self, Self::Error> {
        Calendar::try_new(
            cal.working_days.unwrap_or(WORK_WEEK.to_vec()),
            cal.holidays.unwrap_or_default(),
            cal.extra_working_dates.unwrap_or_default(),
        )
    }
}

#[derive(Debug, PartialEq)]
pub struct Error(String);

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl Calendar {
    pub fn try_new(
        working_days: Vec<Weekday>,
        holidays: Vec<NaiveDate>,
        extra_working_dates: Vec<NaiveDate>,
    ) -> Result<Calendar, Error> {
        let holidays: HashSet<_> = holidays.into_iter().collect();
        let extra_working_dates: HashSet<_> = extra_working_dates.into_iter().collect();

        if !holidays.is_disjoint(&extra_working_dates) {
            return Err(Error(String::from("Holidays cannot be extra working days")));
        }

        Ok(Self {
            working_days,
            holidays,
            extra_working_dates,
        })
    }

    pub fn is_business_day(&self, date: &NaiveDate) -> bool {
        self.is_working_day(date) && !self.holidays.contains(&date)
    }

    pub fn roll_forward(&self, date: &NaiveDate) -> NaiveDate {
        let mut result = date.clone();
        while !self.is_business_day(&result) {
            result += Duration::days(1);
        }
        result
    }

    pub fn roll_backward(&self, date: &NaiveDate) -> NaiveDate {
        let mut result = date.clone();
        while !self.is_business_day(&result) {
            result -= Duration::days(1);
        }
        result
    }

    pub fn next_business_day(&self, date: &NaiveDate) -> NaiveDate {
        let mut result = date.clone();
        loop {
            result += Duration::days(1);
            if self.is_business_day(&result) {
                break;
            }
        }
        result
    }

    pub fn previous_business_day(&self, date: &NaiveDate) -> NaiveDate {
        let mut result = date.clone();
        loop {
            result -= Duration::days(1);
            if self.is_business_day(&result) {
                break;
            }
        }
        result
    }

    pub fn add_business_days(&self, date: &NaiveDate, delta: u32) -> NaiveDate {
        let mut result = self.roll_forward(&date);
        for _ in 0..delta {
            result = self.next_business_day(&result);
        }
        result
    }

    pub fn subtract_business_days(&self, date: &NaiveDate, delta: u32) -> NaiveDate {
        let mut result = self.roll_forward(&date);
        for _ in 0..delta {
            result = self.previous_business_day(&result);
        }
        result
    }

    fn is_working_day(&self, date: &NaiveDate) -> bool {
        self.extra_working_dates.contains(date) || self.working_days.contains(&date.weekday())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn holiday_extra_working_overlap_invalid() {
        let hol1 = NaiveDate::from_ymd(2022, 10, 01);
        let hol2 = NaiveDate::from_ymd(2022, 10, 02);
        let extra_working = NaiveDate::from_ymd(2022, 10, 01);
        let cal = Calendar::try_new(WORK_WEEK.to_vec(), vec![hol1, hol2], vec![extra_working]);

        assert_eq!(
            cal,
            Err(Error(String::from("Holidays cannot be extra working days")))
        );
    }

    #[test]
    fn sat_is_not_business() {
        let cal = Calendar::try_new(WORK_WEEK.to_vec(), vec![], vec![]).unwrap();
        let saturday = NaiveDate::from_ymd(2022, 10, 01);

        assert_eq!(cal.is_business_day(&saturday), false);
    }

    #[test]
    fn sat_extra_working_is_business() {
        let saturday = NaiveDate::from_ymd(2022, 10, 01);
        let cal = Calendar::try_new(WORK_WEEK.to_vec(), vec![], vec![saturday]).unwrap();

        assert_eq!(cal.is_business_day(&saturday), true);
    }

    #[test]
    fn mon_is_business() {
        let cal = Calendar::try_new(WORK_WEEK.to_vec(), vec![], vec![]).unwrap();
        let monday = NaiveDate::from_ymd(2022, 10, 03);

        assert_eq!(cal.is_business_day(&monday), true);
    }

    #[test]
    fn mon_holiday_is_not_business() {
        let monday = NaiveDate::from_ymd(2022, 10, 03);
        let cal = Calendar::try_new(WORK_WEEK.to_vec(), vec![monday], vec![]).unwrap();

        assert_eq!(cal.is_business_day(&monday), false);
    }

    #[test]
    fn sat_rolls_forward_to_tues() {
        let sat = NaiveDate::from_ymd(2022, 10, 01);
        let holiday_mon = NaiveDate::from_ymd(2022, 10, 03);
        let cal = Calendar::try_new(WORK_WEEK.to_vec(), vec![holiday_mon], vec![]).unwrap();

        let business_tue = NaiveDate::from_ymd(2022, 10, 04);

        assert_eq!(cal.roll_forward(&sat), business_tue);
    }

    #[test]
    fn mon_rolls_forward_same_day() {
        let mon = NaiveDate::from_ymd(2022, 10, 03);
        let cal = Calendar::try_new(WORK_WEEK.to_vec(), vec![], vec![]).unwrap();

        assert_eq!(cal.roll_forward(&mon), mon);
    }

    #[test]
    fn sun_rolls_backward_to_thu() {
        let sun = NaiveDate::from_ymd(2022, 10, 02);
        let holiday_fri = NaiveDate::from_ymd(2022, 09, 30);
        let cal = Calendar::try_new(WORK_WEEK.to_vec(), vec![holiday_fri], vec![]).unwrap();

        let business_thu = NaiveDate::from_ymd(2022, 09, 29);

        assert_eq!(cal.roll_backward(&sun), business_thu);
    }

    #[test]
    fn mon_rolls_backward_same_day() {
        let mon = NaiveDate::from_ymd(2022, 10, 03);
        let cal = Calendar::try_new(WORK_WEEK.to_vec(), vec![], vec![]).unwrap();

        assert_eq!(cal.roll_backward(&mon), mon);
    }

    #[test]
    fn sat_next_business_is_tues() {
        let sat = NaiveDate::from_ymd(2022, 10, 01);
        let holiday_mon = NaiveDate::from_ymd(2022, 10, 03);
        let cal = Calendar::try_new(WORK_WEEK.to_vec(), vec![holiday_mon], vec![]).unwrap();

        let business_tue = NaiveDate::from_ymd(2022, 10, 04);

        assert_eq!(cal.next_business_day(&sat), business_tue);
    }

    #[test]
    fn mon_next_business_is_tues() {
        let mon = NaiveDate::from_ymd(2022, 10, 03);
        let cal = Calendar::try_new(WORK_WEEK.to_vec(), vec![], vec![]).unwrap();

        let tue = NaiveDate::from_ymd(2022, 10, 04);
        assert_eq!(cal.next_business_day(&mon), tue);
    }

    #[test]
    fn sun_previous_business_is_thu() {
        let sun = NaiveDate::from_ymd(2022, 10, 02);
        let holiday_fri = NaiveDate::from_ymd(2022, 09, 30);
        let cal = Calendar::try_new(WORK_WEEK.to_vec(), vec![holiday_fri], vec![]).unwrap();

        let business_thu = NaiveDate::from_ymd(2022, 09, 29);

        assert_eq!(cal.previous_business_day(&sun), business_thu);
    }

    #[test]
    fn mon_previous_business_is_fri() {
        let mon = NaiveDate::from_ymd(2022, 10, 03);
        let cal = Calendar::try_new(WORK_WEEK.to_vec(), vec![], vec![]).unwrap();

        let fri = NaiveDate::from_ymd(2022, 09, 30);
        assert_eq!(cal.previous_business_day(&mon), fri);
    }

    #[test]
    fn sat_add_2_business_is_thu() {
        let sat = NaiveDate::from_ymd(2022, 10, 01);
        let holiday_tues = NaiveDate::from_ymd(2022, 10, 04);
        let cal = Calendar::try_new(WORK_WEEK.to_vec(), vec![holiday_tues], vec![]).unwrap();

        let business_thu = NaiveDate::from_ymd(2022, 10, 06);

        assert_eq!(cal.add_business_days(&sat, 2), business_thu);
    }

    #[test]
    fn mon_add_2_business_is_wed() {
        let mon = NaiveDate::from_ymd(2022, 10, 03);
        let cal = Calendar::try_new(WORK_WEEK.to_vec(), vec![], vec![]).unwrap();

        let wed = NaiveDate::from_ymd(2022, 10, 05);

        assert_eq!(cal.add_business_days(&mon, 2), wed);
    }

    #[test]
    fn sun_sub_2_business_is_wed() {
        let sun = NaiveDate::from_ymd(2022, 10, 02);
        let holiday_fri = NaiveDate::from_ymd(2022, 09, 30);
        let cal = Calendar::try_new(WORK_WEEK.to_vec(), vec![holiday_fri], vec![]).unwrap();

        let business_wed = NaiveDate::from_ymd(2022, 09, 28);

        assert_eq!(cal.subtract_business_days(&sun, 2), business_wed);
    }

    #[test]
    fn web_sub_2_business_is_mon() {
        let wed = NaiveDate::from_ymd(2022, 10, 05);
        let cal = Calendar::try_new(WORK_WEEK.to_vec(), vec![], vec![]).unwrap();

        let mon = NaiveDate::from_ymd(2022, 10, 03);

        assert_eq!(cal.subtract_business_days(&wed, 2), mon);
    }

    #[test]
    fn parse_yaml() {
        let input = "
            working_days:
                - monday
                - tuesday
                - wednesday
                - thursday
                - friday

            holidays:
              - 2022-01-01
              - 2012-12-25

            extra_working_dates:
              - 2022-11-09
        ";
        let cal: Calendar = serde_yaml::from_str(input).unwrap();

        let expected = Calendar::try_new(
            WORK_WEEK.to_vec(),
            vec![
                NaiveDate::from_ymd(2022, 1, 1),
                NaiveDate::from_ymd(2012, 12, 25),
            ],
            vec![NaiveDate::from_ymd(2022, 11, 9)],
        )
        .unwrap();

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

        let expected = Calendar::try_new(
            WORK_WEEK.to_vec(),
            vec![
                NaiveDate::from_ymd(2022, 1, 1),
                NaiveDate::from_ymd(2012, 12, 25),
            ],
            vec![],
        )
        .unwrap();

        assert_eq!(cal, expected);
    }
}
