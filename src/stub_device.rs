use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

struct MultiharpDeviceInfo<'a> {
    // Amalgamation of device-related information collected from
    // several different API calls, for convenience.
    device_index: u32,

    // MH_GetLibraryVersion
    library_version: &'a str,

    // MH_GetHardwareInfo
    model: &'a str,
    partno: &'a str,
    version: &'a str,

    // MH_GetSerialNumber
    serial_number: &'a str,

    // MH_GetBaseResolution
    base_resolution: f64,
    binsteps: u32,

    // MH_GetNumOfInputChannels
    num_channels: u32,
}

struct StubMultiharpDevice {}

impl StubMultiharpDevice {
    pub fn get_device_info(&self) -> MultiharpDeviceInfo {
        MultiharpDeviceInfo {
            library_version: "1.0",
            device_index: 1,
            model: "Base stub device",
            partno: "one",
            version: "2.0",
            serial_number: "abcd1234",
            base_resolution: 5.0,
            binsteps: 1,
            num_channels: 8,
        }
    }

    pub fn stream_measurement(
        &self,
        measurement_time: &Duration,
        tx_channel: &mpsc::Sender<Vec<u32>>,
    ) {
        let start_time = Instant::now();
        while start_time.elapsed() < *measurement_time {
            tx_channel.send(self.generate_raw_records());
            thread::sleep(Duration::from_millis(100));
        }
    }

    fn generate_raw_records(&self) -> Vec<u32> {
        let mut raw_records = Vec::with_capacity(500000);
        for event_time in 0..500000 {
            raw_records.push(0x02000001 + event_time);
        }
        raw_records
    }
}
