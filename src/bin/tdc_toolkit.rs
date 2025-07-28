use anyhow::{Error, Result, bail};
use clap::{Parser, Subcommand, ValueEnum, ValueHint};
use indicatif::{ProgressBar, ProgressStyle};
use std::fs;
use std::panic::panic_any;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{Duration, Instant};
use strum_macros::Display;

use tdc_toolkit::multiharp::device::{MH160, MH160Device, MH160DeviceConfig};
use tdc_toolkit::multiharp::device_stub::MH160Stub;
use tdc_toolkit::multiharp::recording;

#[derive(Debug, Parser)]
#[command(name = "tdc_toolkit")]
#[command(about = "A CLI for controlling time-to-digital converters such as the PicoQuant MultiHarp.", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Output information about the device, in JSON format. The
    /// schema is device-specific.
    Info {
        /// Multiharp-specfic. When more than one device is connected
        /// to the computer, select the one to connect to.
        #[arg(long, default_value_t = 0)]
        mh_device_index: u8,

        /// The type of device being connected.
        #[arg(long, default_value_t = DeviceType::MH160Device)]
        device_type: DeviceType,
    },

    /// Record the output of a device to a series of Parquet
    /// files.
    ///
    /// Regardless of the input device, all output files contain the
    /// following fields:
    ///
    /// * `channel`: The device channel associated with the
    ///   event. [`arrow::datatypes::DataType::UInt16`]
    ///
    /// * `time_tag`: The monotonic timestamp associated with the
    ///   event, in picoseconds. [`arrow::datatypes::DataType::UInt64`]
    Record {
        /// The directory where output Parquet files will be
        /// written. Must exist before the program is run; this
        /// program will not create the directory.
        #[arg(long, value_hint = ValueHint::DirPath, default_value_os_t = PathBuf::from_str(".").unwrap())]
        output_dir: PathBuf,

        /// Path to the configuration file for the
        /// device. Configuration is device-specific; if this field is
        /// omitted and the device supports it, the device will be
        /// opened without changing its current configuration.
        ///
        /// The configuration format for MultiHarp is documented in
        /// [`MH160DeviceConfig`]. JSON examples are available in the
        /// source distribution's `sample_config` directory.
        #[arg(long, value_hint = ValueHint::FilePath)]
        device_config: Option<PathBuf>,

        /// Multiharp-specfic. When more than one device is connected
        /// to the computer, select the one to connect to.
        #[arg(long, default_value_t = 0)]
        mh_device_index: u8,

        /// The duration of time to measure. Can be specified in any
        /// format allowed by [`humantime::parse_duration`].
        #[arg(long)]
        duration: humantime::Duration,

        /// The type of device being connected.
        #[arg(long, default_value_t = DeviceType::MH160Device)]
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
    MH160Device,
    MH160StubGenerator,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Info {
            mh_device_index,
            device_type,
        } => {
            let device = match device_type {
                DeviceType::MH160Device => Ok::<Box<dyn MH160>, Error>(Box::new(
                    MH160Device::from_current_config(mh_device_index)?,
                )),
                DeviceType::MH160StubGenerator => Ok(Box::new(MH160Stub {}) as Box<dyn MH160>),
            }?;
            println!(
                "{}",
                &serde_json::to_string_pretty(&device.get_device_info()?)?
            );
            Ok(())
        }
        Command::Record {
            output_dir,
            device_config,
            mh_device_index,
            duration,
            device_type,
            name,
        } => {
            let device = match device_type {
                DeviceType::MH160Device => {
                    let unboxed_device = match device_config {
                        Some(device_config) => {
                            let config: MH160DeviceConfig =
                                serde_json::from_str(fs::read_to_string(device_config)?.as_str())?;
                            MH160Device::from_config(mh_device_index, config)?
                        }
                        None => MH160Device::from_current_config(mh_device_index)?,
                    };
                    Ok::<Arc<dyn MH160>, Error>(Arc::new(unboxed_device) as Arc<(dyn MH160)>)
                }
                DeviceType::MH160StubGenerator => Ok(Arc::new(MH160Stub {}) as Arc<(dyn MH160)>),
            }?;

            let recording_failed = Arc::new(AtomicBool::new(false));
            let recording_failed_thread_clone = recording_failed.clone();

            let recording_thread = thread::Builder::new()
                .name("recording_thread".into())
                .spawn(move || -> Result<()> {
                    if let Err(recording_error) = recording::record_multiharp_to_parquet(
                        device.clone(),
                        output_dir,
                        *duration,
                        name,
                    ) {
                        recording_failed_thread_clone.store(true, Ordering::Relaxed);
                        panic_any(recording_error);
                    }
                    Ok(())
                })?;

            let progress_bar = ProgressBar::new(duration.as_millis().try_into()?)
                .with_style(ProgressStyle::with_template(
                    "[{elapsed_precise}] {bar:40} {msg}",
                )?)
                .with_message("Recording...");
            let start_time = Instant::now();
            while start_time.elapsed() < *duration {
                if recording_failed.load(Ordering::Relaxed) {
                    progress_bar.finish_with_message("Recording failed");
                    break;
                }

                progress_bar.set_position(start_time.elapsed().as_millis().try_into()?);
                thread::sleep(Duration::from_millis(100));
            }

            let recording_thread_name = recording_thread
                .thread()
                .name()
                .unwrap_or("unnamed")
                .to_owned();
            if let Err(recording_panic) = recording_thread.join() {
                if let Ok(recording_panic_anyhow) = recording_panic.downcast::<Error>() {
                    bail!(
                        "Error returned from thread {}:\n{:?}",
                        recording_thread_name,
                        recording_panic_anyhow
                    );
                }
                panic!(
                    "Failed downcast of thread {recording_thread_name} error result to anyhow::Error. This should not happen. Threads in this application should always return anyhow::Error."
                );
            }

            progress_bar.finish_with_message("Recording complete");
            Ok(())
        }
    }
}
