use business::Calendar;
use chrono::naive::NaiveDate;

fn main() {
    let yml = std::fs::read_to_string("examples/basic/cal.yml").unwrap();
    let cal: Calendar = serde_yaml::from_str(&yml).unwrap();

    let xmas = NaiveDate::from_ymd(2017, 12, 25);
    let last_business_day = cal.roll_backward(xmas);

    println!(
        "The last business day before Christmas is: {}",
        last_business_day,
    );

    // Delivery takes 2 business days.
    // Accounts for Christmas, Boxing Day, and the weekend.
    println!(
        "Your package will arrive on: {}",
        cal.add_business_days(last_business_day, 2),
    )
}
