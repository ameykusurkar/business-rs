use chrono::{naive::NaiveDate, Datelike, Weekday};

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
}
