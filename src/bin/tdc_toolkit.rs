use anyhow::{anyhow, Error, Result};
use clap::{Parser, Subcommand, ValueEnum, ValueHint};
use indicatif::{ProgressBar, ProgressStyle};
use std::fs;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};
use strum_macros::Display;

use tdc_toolkit::multiharp_device;
use tdc_toolkit::multiharp_device::MultiharpDevice;
use tdc_toolkit::multiharp_device_stub;
use tdc_toolkit::recording;

#[derive(Debug, Parser)]
#[command(name = "tdc_toolkit")]
#[command(about = "A CLI for controlling time-to-digital converters such as the PicoQuant MultiHarp.", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Record the output of a device to a series of Parquet
    /// files. Regardless of the input device, all output files contain the following fields:
    ///
    /// * `channel`: The device channel associated with the
    ///   event. [`arrow::datatypes::DataType::UInt16`]
    /// * `time_tag`: The monotonic timestamp associated with the
    ///   event, in picoseconds. [`arrow::datatypes::DataType::UInt64`]
    Record {
        /// The directory where output Parquet files will be
        /// written. Must exist before the program is run; this
        /// program will not create the directory.
        #[arg(long, value_hint = ValueHint::DirPath, default_value_os_t = PathBuf::from_str(".").unwrap())]
        output_dir: PathBuf,

        /// Path to the configuration file for the
        /// device. Configuration is device-specific; if a device
        /// requires no configuration, this field may be omitted.
        #[arg(long, value_hint = ValueHint::FilePath, default_value_os_t = PathBuf::from_str("./conf.json").unwrap())]
        device_config: PathBuf,

        /// Multiharp-specfic. When more than one device is connected
        /// to the computer, select the one to connect to.
        #[arg(long, default_value_t = 0)]
        mh_device_index: u8,

        /// The duration of time to measure. Can be specified in any
        /// format allowed by [`humantime::parse_duration`].
        #[arg(long)]
        duration: humantime::Duration,

        /// The type of device being connected.
        #[arg(long, default_value_t = DeviceType::Multiharp160)]
        device_type: DeviceType,

        /// A string that will be used as part of the output filename,
        /// to help distinguish this recording session from other
        /// sessions in the same directory.
        #[arg(long, default_value_t = String::from("record"))]
        name: String,
    },
}

#[derive(Copy, Clone, Debug, Display, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
#[strum(serialize_all = "kebab-case")]
enum DeviceType {
    Multiharp160,
    Multiharp160StubGenerator,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Record {
            output_dir,
            device_config,
            mh_device_index,
            duration,
            device_type,
            name,
        } => {
            let device = match device_type {
                DeviceType::Multiharp160 => {
                    let config: multiharp_device::MultiharpDeviceConfig =
                        serde_json::from_str(fs::read_to_string(device_config)?.as_str())?;
                    let device =
                        multiharp_device::Multiharp160::from_config(mh_device_index, config)?;
                    let device_arc = Arc::new(device) as Arc<(dyn MultiharpDevice + Send + Sync)>;
                    Ok::<Arc<dyn MultiharpDevice + Send + Sync>, Error>(device_arc)
                }
                // TODO + Send + Sync should be part of the
                // MultiharpDevice trait signature. Although is it
                // really a good idea to make this Sync?
                DeviceType::Multiharp160StubGenerator => {
                    Ok(Arc::new(multiharp_device_stub::Multiharp160Stub {})
                        as Arc<(dyn MultiharpDevice + Send + Sync)>)
                }
            }?;
            let recording_thread = thread::spawn(move || -> Result<()> {
                recording::record_multiharp_to_parquet(
                    device.clone(),
                    &output_dir,
                    *duration,
                    &name,
                )?;
                Ok(())
            });

            let progress_bar = ProgressBar::new(duration.as_millis().try_into()?)
                .with_style(ProgressStyle::with_template(
                    "[{elapsed_precise}] {bar:40} {msg}",
                )?)
                .with_message("Recording...");
            let start_time = Instant::now();
            while start_time.elapsed() < *duration {
                progress_bar.set_position(start_time.elapsed().as_millis().try_into()?);
                thread::sleep(Duration::from_millis(100));
            }

            // TODO this isn't sufficient, see note in recording.rs
            if let Err(recording_error) = recording_thread.join() {
                if let Some(recording_anyhow_error) = recording_error.downcast_ref::<Error>() {
                    return Err(anyhow!("Something happened {}", recording_anyhow_error));
                }
            }

            progress_bar.finish_with_message("Recording complete");
            Ok(())
        }
    }
}
