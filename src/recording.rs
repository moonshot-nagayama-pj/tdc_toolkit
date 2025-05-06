use anyhow::{anyhow, Error, Result};
use std::path::Path;
use std::sync::{mpsc, Arc};
use std::thread;
use std::time::Duration;

use crate::multiharp_device::MultiharpDevice;
use crate::parquet_writer;
use crate::tttr_record;

pub fn record_multiharp_to_parquet(
    device: Arc<(dyn MultiharpDevice + Send + Sync)>,
    output_dir: &Path,
    duration: Duration,
    name: &str,
) -> Result<()> {
    let (raw_tx_channel, raw_rx_channel) = mpsc::channel();
    let (processed_tx_channel, processed_rx_channel) = mpsc::channel();

    let device_thread = thread::spawn(move || -> Result<()> {
        device.stream_measurement(&duration, raw_tx_channel)?;
        Ok(())
    });

    let processor_thread = thread::spawn(move || -> Result<()> {
        let mut processor = tttr_record::T2RecordChannelProcessor::new();
        processor.process(raw_rx_channel, processed_tx_channel)?;
        Ok(())
    });

    let writer = parquet_writer::T2RecordParquetWriter::new();
    writer.write(processed_rx_channel, output_dir, name)?;
    // TODO this isn't sufficient -- need to join all threads and
    // collect errors, as well as deal with non-Anyhow errors
    if let Err(device_error) = device_thread.join() {
        if let Some(device_anyhow_error) = device_error.downcast_ref::<Error>() {
            return Err(anyhow!("Something happened {}", device_anyhow_error));
        }
    }
    if let Err(processor_error) = processor_thread.join() {
        if let Some(processor_anyhow_error) = processor_error.downcast_ref::<Error>() {
            return Err(anyhow!("Something happened {}", processor_anyhow_error));
        }
    }
    Ok(())
}
