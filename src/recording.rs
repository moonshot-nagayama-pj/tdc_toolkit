use anyhow::{anyhow, Error, Result};
use std::path::Path;
use std::sync::{mpsc, Arc};
use std::thread;
use std::time::Duration;

use crate::multiharp_device::MultiharpDevice;
use crate::parquet_writer;
use crate::tttr_record;

fn join_and_collect_thread_errors<T>(handles: Vec<thread::JoinHandle<T>>) -> Option<Error> {
    let mut error_str = String::from("");
    for handle in handles {
        let thread_name = handle.thread().name().unwrap_or("unnamed").to_owned();
        if let Err(error) = handle.join() {
            if let Some(anyhow_error) = error.downcast_ref::<Error>() {
                error_str.push_str(&format!(
                    "Error returned from thread {}:\n{:?}----------\n",
                    thread_name, anyhow_error
                ));
            } else {
                panic!("Failed downcast to anyhow::Error. This should not happen. Threads in this application should always return anyhow::Error.");
            }
        }
    }
    if error_str.is_empty() {
        return None;
    }
    Some(anyhow!("Error in one or more threads.").context(error_str))
}

pub fn record_multiharp_to_parquet(
    device: Arc<(dyn MultiharpDevice + Send + Sync)>,
    output_dir: &Path,
    duration: Duration,
    name: &str,
) -> Result<()> {
    let (raw_tx_channel, raw_rx_channel) = mpsc::channel();
    let (processed_tx_channel, processed_rx_channel) = mpsc::channel();

    let mut handles = Vec::new();

    let device_thread =
        thread::Builder::new()
            .name("device_thread".into())
            .spawn(move || -> Result<()> {
                device.stream_measurement(&duration, raw_tx_channel)?;
                Ok(())
            })?;
    handles.push(device_thread);

    let processor_thread = thread::Builder::new()
        .name("processor_thread".into())
        .spawn(move || -> Result<()> {
            let mut processor = tttr_record::T2RecordChannelProcessor::new();
            processor.process(raw_rx_channel, processed_tx_channel)?;
            Ok(())
        })?;
    handles.push(processor_thread);

    let writer = parquet_writer::T2RecordParquetWriter::new();
    writer.write(processed_rx_channel, output_dir, name)?;

    // TODO still need to ensure clean signaling between threads if
    // one shuts down early to stop channel from running forever
    match join_and_collect_thread_errors(handles) {
        None => Ok(()),
        Some(error) => Err(error),
    }
}
