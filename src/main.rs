use chrono::{Local, Timelike};
mod model;

/// Returns whether the current local time falls within the configured sleep window.
///
/// Change these two values to set the range. The start is inclusive and the end is
/// exclusive, so `23:00..07:00` means from 23:00 through 06:59.
const SLEEP_START_HOUR  : u32 = 21;
const SLEEP_START_MIN   : u32 = 30;
const SLEEP_END_HOUR    : u32 = 06;
const SLEEP_END_MIN     : u32 = 00;

fn is_sleep_time() -> bool {
    let now = Local::now();

    let after_sleep_start = now.hour() > SLEEP_START_HOUR
        || (now.hour() == SLEEP_START_HOUR && now.minute() >= SLEEP_START_MIN);

    let before_sleep_end = now.hour() < SLEEP_END_HOUR
    || (now.hour() == SLEEP_END_HOUR && now.minute() < SLEEP_END_MIN);


    if SLEEP_START_HOUR < SLEEP_END_HOUR
        || (SLEEP_START_HOUR == SLEEP_END_HOUR && SLEEP_START_MIN <= SLEEP_END_MIN)
    {
        after_sleep_start && before_sleep_end
    } else {
        // A range such as 23:00..07:00 crosses midnight.
        after_sleep_start || before_sleep_end
    }
}


fn main() {
    let now = Local::now();
    println!("Hour: {}", now.hour());
    println!("Is sleep time: {}", is_sleep_time());

    model::image_test()
}
