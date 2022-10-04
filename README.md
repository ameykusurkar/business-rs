[![Workflow Status](https://github.com/ameykusurkar/business-rs/workflows/main/badge.svg)](https://github.com/ameykusurkar/business-rs/actions?query=workflow%3A%22main%22)

# business

A crate for doing business day calculations. It is a Rust implementation of GoCardless'
[business](https://github.com/gocardless/business) Ruby gem.

Let's dive right in with an example. For more details, see [`Calendar`].

```rust
use chrono::NaiveDate;

let xmas = NaiveDate::from_ymd(2020, 12, 25); // Friday

let cal = business::Calendar::with_holidays(&[xmas]);

assert_eq!(cal.is_business_day(xmas), false);

// The earliest business day
assert_eq!(cal.roll_forward(xmas), NaiveDate::from_ymd(2020, 12, 28));

let xmas_eve = NaiveDate::from_ymd(2020, 12, 24);
assert_eq!(cal.is_business_day(xmas_eve), true);

// Skips over weekend and business holidays
assert_eq!(cal.add_business_days(xmas_eve, 2), NaiveDate::from_ymd(2020, 12, 29));
```

## Building a `Calendar` from YAML

The YAML has to be in the following format:
```yaml
# Defaults to Mon-Fri is omitted
working_days:
  - monday
  - tuesday
  - wednesday
  - thursday
  - friday
# ISO 8601 dates, defaults to no holidays if omitted
holidays:
  - 2017-12-25
  - 2017-12-26
```
A calendar can be built as such:
```rust
let yml = std::fs::read_to_string("examples/basic/cal.yml").unwrap();
let cal: Calendar = serde_yaml::from_str(&yml).unwrap();
```

License: MIT