use std::{
    process::{Child, Command},
    thread::sleep,
    time::Duration
};
use burn_flex::Flex;
use chrono::{Local, Timelike};

use hexawaker::{
    sleep_person_yolo26n,
    model,
    camera::{init_camera, Stream},
    debug_println,
};

const SLEEP_START_HOUR  : u32 = 00;
const SLEEP_START_MIN   : u32 = 30;
const SLEEP_END_HOUR    : u32 = 06;
const SLEEP_END_MIN     : u32 = 00;
const INTERVAL          : Duration = Duration::from_millis(500);


/// Returns whether the current local time falls within the configured sleep window.
///
/// Change these two values to set the range. The start is inclusive and the end is
/// exclusive, so `23:00..07:00` means from 23:00 through 06:59.
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


fn start_alarm() -> Child {
    Command::new("sh")
        .args([
            "-c",
            "while true; do aplay /home/mirko/Downloads/prueba/sample-9s.wav; done",
        ])
        .spawn()
        .expect("Failed to start alarm")
}


fn stop_alarm(alarm: &mut Option<Child>) {
    if let Some(mut process) = alarm.take() {
        let _ = process.kill();
    }
}


fn main() {
    type Backend = Flex;
    let device = Default::default();
    
    let model = sleep_person_yolo26n::Model::<Backend>::default();
    debug_println!("Model loaded!");

    let camera = init_camera();
    let stream = Stream::new(camera);
    // Sleep 2 seconds to ensure stream initialization
    sleep(Duration::from_secs(2));

    let mut counter = 0;
    let mut alarm: Option<Child> = None;
    loop {
        if is_sleep_time() {
            counter = 0;
            stop_alarm(&mut alarm);
            debug_println!("Its sleep time");
            sleep(INTERVAL);
            continue
        }
    
        let frame = stream.get_last_frame();
        debug_println!("Captured frame: {} bytes", frame.len());

        let image = image::load_from_memory(frame.as_slice())
            .expect("Failed to decode JPEG")
            .to_rgb8();

        let detections = model::analyze_image(&device, &model, image.clone());
        debug_println!("Got {} detections", detections.len());

        if detections.is_empty() {
            counter = 0;
            stop_alarm(&mut alarm);
            sleep(INTERVAL);
            continue
        }

        counter += 1;

        if counter >= 5 && alarm.is_none() {
            alarm = Some(start_alarm());
        }

        sleep(INTERVAL);
    }
}
