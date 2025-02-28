use std::path::Path;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use _mhtk_rs::mhlib_wrapper_header::Edge;
use _mhtk_rs::multiharp_device::{
    Multiharp160, MultiharpDevice, MultiharpDeviceConfig, MultiharpDeviceInputChannelConfig,
};
use _mhtk_rs::parquet_writer;
use _mhtk_rs::tttr_record;

fn process_measurements() {
    let (raw_tx_channel, raw_rx_channel) = mpsc::channel();
    let (processed_tx_channel, processed_rx_channel) = mpsc::channel();

    let config = MultiharpDeviceConfig {
        sync_channel: None,
        input_channels: vec![
            MultiharpDeviceInputChannelConfig {
                id: 0,
                edge_trigger_level: 250, // mV
                edge_trigger: Edge::Falling,
                offset: 0, // picoseconds
            },
            MultiharpDeviceInputChannelConfig {
                id: 1,
                edge_trigger_level: 250, // mV
                edge_trigger: Edge::Falling,
                offset: 0, // picoseconds
            },
            MultiharpDeviceInputChannelConfig {
                id: 2,
                edge_trigger_level: 250, // mV
                edge_trigger: Edge::Falling,
                offset: 0, // picoseconds
            },
            MultiharpDeviceInputChannelConfig {
                id: 3,
                edge_trigger_level: 250, // mV
                edge_trigger: Edge::Falling,
                offset: 0, // picoseconds
            },
        ],
    };

    let device = Multiharp160::from_config(0, config);
    println!("{:?}", device.get_device_info());

    let device_thread = thread::spawn(move || {
        device.stream_measurement(&Duration::from_millis(100), raw_tx_channel);
    });

    let processor_thread = thread::spawn(|| {
        let mut processor = tttr_record::T2RecordChannelProcessor::new();
        processor.process(raw_rx_channel, processed_tx_channel);
    });

    let writer = parquet_writer::T2RecordParquetWriter::new();
    writer
        .write(processed_rx_channel, Path::new("."), "test")
        .unwrap();
    device_thread.join().unwrap();
    processor_thread.join().unwrap();
}

fn main() {
    process_measurements();
}
