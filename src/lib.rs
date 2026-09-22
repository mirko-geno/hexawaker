#[macro_export]
macro_rules! debug_println {
    ($($arg:tt)*) => {
        #[cfg(debug_assertions)]
        println!($($arg)*);
    };
}

pub mod sleep_person_yolo26n {
    include!(concat!(env!("OUT_DIR"), "/model/sleep_person_yolo26n-v2.rs"));
}
pub mod model;
pub mod camera;