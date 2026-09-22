use std::{
    thread,
    sync::{Arc, Mutex}
};
use v4l::{
    buffer::Type,
    device::Device,
    io::traits::CaptureStream,
    video::Capture,
    prelude::*
};


pub struct Stream {
    latest_frame: Arc<Mutex<Vec<u8>>>,
}

impl Stream {
    pub fn new(camera: Device) -> Self {
        let mut stream = MmapStream::with_buffers(
            &camera,
            Type::VideoCapture,
            4,
        )
        .expect("Failed to create camera stream");

        let latest_frame = Arc::new(Mutex::new(Vec::new()));
        let latest_frame_thread = Arc::clone(&latest_frame);

        thread::spawn(move || {
            loop {
                match stream.next() {
                    Ok((frame, _)) => {
                        let mut latest = latest_frame_thread
                            .lock()
                            .expect("Stream mutex poisoned");

                        *latest = frame.to_vec();
                    }

                    Err(_error) => {
                        debug_println!("Stream error: {_error}");
                    }
                }
            }
        });

        Self { latest_frame }
    }

    pub fn get_last_frame(&self) -> Vec<u8> {
        self.latest_frame
            .lock()
            .expect("Stream mutex poisoned")
            .clone()
    }
}


pub fn init_camera() -> Device {
    let camera = Device::with_path("/dev/video0")
        .expect("Failed to open camera");

    let mut format = camera
        .format()
        .expect("Failed to get camera format");

    format.width = 640;
    format.height = 480;
    format.fourcc = v4l::FourCC::new(b"MJPG");

    camera
        .set_format(&format)
        .expect("Failed to set camera format");

    debug_println!("Camera initialized:");
    debug_println!("  Resolution: {}x{}", format.width, format.height);
    debug_println!("  Format: {:?}", format.fourcc.str().unwrap());

    camera
}
