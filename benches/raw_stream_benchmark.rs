use criterion::{criterion_group, criterion_main, Criterion};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use _mhtk_rs::stub_device;
use _mhtk_rs::tttr_record;

fn process_measurements() {
    let (raw_tx_channel, raw_rx_channel) = mpsc::channel();
    let (processed_tx_channel, processed_rx_channel) = mpsc::channel();

    let device_handle = thread::spawn(|| {
        let device = stub_device::StubMultiharpDevice {};
        device.stream_measurement(&Duration::from_millis(1000), raw_tx_channel);
    });

    let processor_handle = thread::spawn(|| {
        let mut processor = tttr_record::T2RecordChannelProcessor::new();
        processor.process(raw_rx_channel, processed_tx_channel);
    });

    let mut total_messages = 0u64;
    for processed_messages in processed_rx_channel {
        total_messages += processed_messages.len() as u64;
    }
    println!("{}", total_messages);

    device_handle.join().unwrap();
    processor_handle.join().unwrap();
}

pub fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("raw_stream", |b| b.iter(|| process_measurements()));
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(10);
    targets = criterion_benchmark
}
criterion_main!(benches);
