use anyhow::Result;
use std::panic::panic_any;
use std::path::PathBuf;
use std::sync::{Arc, mpsc};
use std::thread;
use std::time::Duration;

use super::device::MH160;
use super::tttr_record;
use crate::output::parquet;
use crate::util::join_and_collect_thread_errors;

pub fn record_multiharp_to_parquet(
    device: Arc<dyn MH160 + Send + Sync>,
    output_dir: PathBuf,
    duration: Duration,
    name: String,
) -> Result<()> {
    let (raw_send_channel, raw_receive_channel) = mpsc::channel();
    let (processed_send_channel, processed_receive_channel) = mpsc::channel();

    let mut handles = Vec::new();

    let device_thread =
        thread::Builder::new()
            .name("device_thread".into())
            .spawn(move || -> Result<()> {
                if let Err(error) = device.stream_measurement(&duration, raw_send_channel) {
                    panic_any(error);
                }
                Ok(())
            })?;
    handles.push(device_thread);

    let processor_thread = thread::Builder::new()
        .name("processor_thread".into())
        .spawn(move || -> Result<()> {
            let mut processor = tttr_record::T2RecordChannelProcessor::new();
            if let Err(error) = processor.process(raw_receive_channel, processed_send_channel) {
                panic_any(error);
            }
            Ok(())
        })?;
    handles.push(processor_thread);

    let writer_thread =
        thread::Builder::new()
            .name("writer_thread".into())
            .spawn(move || -> Result<()> {
                let writer = parquet::TimeTagStreamParquetWriter::new();
                if let Err(error) = writer.write(processed_receive_channel, &output_dir, &name) {
                    panic_any(error);
                }
                Ok(())
            })?;
    handles.push(writer_thread);

    join_and_collect_thread_errors(handles)
}
