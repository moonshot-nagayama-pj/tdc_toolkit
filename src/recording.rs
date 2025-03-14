use std::time::Duration;
use std::sync::{mpsc,Arc};
use std::path::PathBuf;
use std::thread;

use crate::multiharp_device::MultiharpDevice;
use crate::parquet_writer;
use crate::tttr_record;

pub fn record_multiharp_to_parquet(device: Arc<(dyn MultiharpDevice + Send + Sync)>, output_dir: &PathBuf, duration: Duration, name: &str) {
    let (raw_tx_channel, raw_rx_channel) = mpsc::channel();
    let (processed_tx_channel, processed_rx_channel) = mpsc::channel();

    let device_thread = thread::spawn(move || {
        device.stream_measurement(&duration, raw_tx_channel);
    });

    let processor_thread = thread::spawn(move || {
        let mut processor = tttr_record::T2RecordChannelProcessor::new();
        processor.process(raw_rx_channel, processed_tx_channel);
    });

    let writer = parquet_writer::T2RecordParquetWriter::new();
    writer
        .write(processed_rx_channel, output_dir, name)
        .unwrap();
    device_thread.join().unwrap();
    processor_thread.join().unwrap();
}
