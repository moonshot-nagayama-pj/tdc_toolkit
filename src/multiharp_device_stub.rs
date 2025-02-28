use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use crate::mhlib_wrapper_header;
use crate::multiharp_device;

pub struct Multiharp160Stub {}

impl Multiharp160Stub {
    fn generate_raw_records(&self) -> Vec<u32> {
        let mut raw_records = Vec::with_capacity(mhlib_wrapper_header::TTREADMAX);
        for event_time in 0..raw_records.capacity() as u32 {
            raw_records.push(0x02000001 + event_time);
        }
        raw_records
    }
}

impl multiharp_device::MultiharpDevice for Multiharp160Stub {
    fn get_device_info(&self) -> multiharp_device::MultiharpDeviceInfo {
        multiharp_device::MultiharpDeviceInfo {
            library_version: "1.0".to_string(),
            device_index: 1,
            model: "Base stub device".to_string(),
            partno: "one".to_string(),
            version: "2.0".to_string(),
            serial_number: "abcd1234".to_string(),
            base_resolution: 5.0,
            binsteps: 1,
            num_channels: 8,
        }
    }

    fn stream_measurement(&self, measurement_time: &Duration, tx_channel: mpsc::Sender<Vec<u32>>) {
        let start_time = Instant::now();
        while start_time.elapsed() < *measurement_time {
            tx_channel
                .send(self.generate_raw_records())
                .expect("send raw_records to channel failed");
            thread::sleep(Duration::from_millis(100));
        }
    }
}
