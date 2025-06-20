use anyhow::Result;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use super::device::{MH160, MH160DeviceInfo};

pub struct MH160Stub {}

impl MH160Stub {
    fn generate_raw_records() -> Vec<u32> {
        let capacity = 1u32;
        let mut raw_records = Vec::with_capacity(capacity as usize); // Can be up to mhlib_wrapper_header::TTREADMAX
        for event_time in 0..capacity {
            raw_records.push(0x0200_0001 + event_time);
        }
        raw_records
    }
}

impl MH160 for MH160Stub {
    fn get_device_info(&self) -> Result<MH160DeviceInfo> {
        Ok(MH160DeviceInfo {
            library_version: "1.0".to_string(),
            device_index: 1,
            model: "Base stub device".to_string(),
            partno: "one".to_string(),
            version: "2.0".to_string(),
            serial_number: "abcd1234".to_string(),
            base_resolution: 5.0,
            binsteps: 1,
            num_channels: 8,
        })
    }

    fn stream_measurement(
        &self,
        measurement_time: &Duration,
        tx_channel: mpsc::Sender<Vec<u32>>,
    ) -> Result<()> {
        let start_time = Instant::now();
        while start_time.elapsed() < *measurement_time {
            tx_channel.send(MH160Stub::generate_raw_records())?;
            thread::sleep(Duration::from_millis(100));
        }
        Ok(())
    }
}
