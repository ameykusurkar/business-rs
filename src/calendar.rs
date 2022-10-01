use chrono::{naive::NaiveDate, Datelike, Duration, Weekday};

pub const WORK_WEEK: [Weekday; 5] = [
    Weekday::Mon,
    Weekday::Tue,
    Weekday::Wed,
    Weekday::Thu,
    Weekday::Fri,
];

pub struct Calendar {
    working_days: Vec<Weekday>,
    holidays: Vec<NaiveDate>,
    extra_working_dates: Vec<NaiveDate>,
}

impl Calendar {
    pub fn new(
        working_days: Vec<Weekday>,
        holidays: Vec<NaiveDate>,
        extra_working_dates: Vec<NaiveDate>,
    ) -> Calendar {
        // TODO: Ensure that holidays an extra_working_dates don't overlap
        Self {
            working_days,
            holidays,
            extra_working_dates,
        }
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

    fn is_working_day(&self, date: &NaiveDate) -> bool {
        self.extra_working_dates.contains(date) || self.working_days.contains(&date.weekday())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sat_is_not_business() {
        let cal = Calendar::new(WORK_WEEK.to_vec(), vec![], vec![]);
        let saturday = NaiveDate::from_ymd(2022, 10, 01);

        assert_eq!(cal.is_business_day(&saturday), false);
    }

    #[test]
    fn sat_extra_working_is_business() {
        let saturday = NaiveDate::from_ymd(2022, 10, 01);
        let cal = Calendar::new(WORK_WEEK.to_vec(), vec![], vec![saturday]);

        assert_eq!(cal.is_business_day(&saturday), true);
    }

    #[test]
    fn mon_is_business() {
        let cal = Calendar::new(WORK_WEEK.to_vec(), vec![], vec![]);
        let monday = NaiveDate::from_ymd(2022, 10, 03);

        assert_eq!(cal.is_business_day(&monday), true);
    }

    #[test]
    fn mon_holiday_is_not_business() {
        let monday = NaiveDate::from_ymd(2022, 10, 03);
        let cal = Calendar::new(WORK_WEEK.to_vec(), vec![monday], vec![]);

        assert_eq!(cal.is_business_day(&monday), false);
    }

    #[test]
    fn sat_rolls_forward_to_tues() {
        let sat = NaiveDate::from_ymd(2022, 10, 01);
        let holiday_mon = NaiveDate::from_ymd(2022, 10, 03);
        let cal = Calendar::new(WORK_WEEK.to_vec(), vec![holiday_mon], vec![]);

        let business_tue = NaiveDate::from_ymd(2022, 10, 04);

        assert_eq!(cal.roll_forward(&sat), business_tue);
    }

    #[test]
    fn mon_rolls_forward_same_day() {
        let mon = NaiveDate::from_ymd(2022, 10, 03);
        let cal = Calendar::new(WORK_WEEK.to_vec(), vec![], vec![]);

        assert_eq!(cal.roll_forward(&mon), mon);
    }

    #[test]
    fn sun_rolls_backward_to_thu() {
        let sun = NaiveDate::from_ymd(2022, 10, 02);
        let holiday_fri = NaiveDate::from_ymd(2022, 09, 30);
        let cal = Calendar::new(WORK_WEEK.to_vec(), vec![holiday_fri], vec![]);

        let business_thu = NaiveDate::from_ymd(2022, 09, 29);

        assert_eq!(cal.roll_backward(&sun), business_thu);
    }

    #[test]
    fn mon_rolls_backward_same_day() {
        let mon = NaiveDate::from_ymd(2022, 10, 03);
        let cal = Calendar::new(WORK_WEEK.to_vec(), vec![], vec![]);

        assert_eq!(cal.roll_backward(&mon), mon);
    }
}
