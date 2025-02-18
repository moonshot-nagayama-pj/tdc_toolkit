use std::path::Path;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use _mhtk_rs::parquet_writer;
use _mhtk_rs::stub_device;
use _mhtk_rs::tttr_record;

fn process_measurements() {
    let (raw_tx_channel, raw_rx_channel) = mpsc::channel();
    let (processed_tx_channel, processed_rx_channel) = mpsc::channel();

    let device_handle = thread::spawn(|| {
        let device = stub_device::StubMultiharpDevice {};
        device.stream_measurement(&Duration::from_millis(100), raw_tx_channel);
    });

    let processor_handle = thread::spawn(|| {
        let mut processor = tttr_record::T2RecordChannelProcessor::new();
        processor.process(raw_rx_channel, processed_tx_channel);
    });

    let writer = parquet_writer::T2RecordParquetWriter::new();
    writer.write(processed_rx_channel, Path::new("."), "test");
    device_handle.join().unwrap();
    processor_handle.join().unwrap();
}

fn main() {
    process_measurements();
}
