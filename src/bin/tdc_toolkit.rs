use anyhow::{Context, Error, Result, bail};
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

use tdc_toolkit::multiharp::device::{MH160, MH160Device};
use tdc_toolkit::multiharp::device_stub::MH160Stub;

#[cfg(feature = "multiharp")]
use tdc_toolkit::multiharp::mhlib_wrapper::real::MhlibWrapperReal;

use tdc_toolkit::multiharp::mhlib_wrapper::stub::MhlibWrapperStub;
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
    /// Output information about the device, in JSON format. The schema is device-specific.
    Info {
        /// MultiHarp-specfic. When more than one device is connected
        /// to the computer, select the one to connect to.
        #[arg(long, default_value_t = 0)]
        mh_device_index: u8,

        /// MultiHarp-specific. Choose between implementations of the wrapper of PicoQuant's proprietary MultiHarp control library.
        #[arg(long, default_value_t = MhWrapperImplementation::default())]
        mh_wrapper_implementation: MhWrapperImplementation,

        /// The type of device being connected.
        #[arg(long, default_value_t = DeviceType::MH160Device)]
        device_type: DeviceType,
    },

    /// Record the output of a device to a series of Parquet files.
    ///
    /// Regardless of the input device, all output files contain the following fields:
    ///
    /// * `channel`: The device channel associated with the event. [`arrow::datatypes::DataType::UInt16`]
    ///
    /// * `time_tag`: The monotonic timestamp associated with the event, in picoseconds. [`arrow::datatypes::DataType::UInt64`]
    Record {
        /// The directory where output Parquet files will be written. Must exist before the program is run; this program will not create the directory.
        #[arg(long, value_hint = ValueHint::DirPath, default_value_os_t = PathBuf::from_str(".").unwrap())]
        output_dir: PathBuf,

        /// Path to the configuration file for the device. Configuration is device-specific; if this field is omitted and the device supports it, the device will be opened without changing its current configuration.
        ///
        /// The configuration format for MultiHarp is documented in [`MH160DeviceConfig`]. JSON examples are available in the source distribution's `sample_config` directory.
        #[arg(long, value_hint = ValueHint::FilePath)]
        device_config: Option<PathBuf>,

        /// MultiHarp-specfic. When more than one device is connected to the computer, select the one to connect to.
        #[arg(long, default_value_t = 0)]
        mh_device_index: u8,

        /// MultiHarp-specific. Choose between implementations for the wrapper of PicoQuant's proprietary MultiHarp control library.
        #[arg(long, default_value_t = MhWrapperImplementation::default())]
        mh_wrapper_implementation: MhWrapperImplementation,

        /// The duration of time to measure. Can be specified in any format allowed by [`humantime::parse_duration`].
        #[arg(long)]
        duration: humantime::Duration,

        /// The type of device being connected.
        #[arg(long, default_value_t = DeviceType::MH160Device)]
        device_type: DeviceType,

        /// A string that will be used as part of the output filename, to help distinguish this recording session from other sessions in the same directory.
        #[arg(long, default_value_t = String::from("record"))]
        name: String,
    },
}

#[cfg(feature = "multiharp")]
#[derive(Copy, Clone, Debug, Default, Display, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
#[strum(serialize_all = "kebab-case")]
enum MhWrapperImplementation {
    #[default]
    Real,
    Stub,
}

#[cfg(not(feature = "multiharp"))]
#[derive(Copy, Clone, Debug, Default, Display, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
#[strum(serialize_all = "kebab-case")]
enum MhWrapperImplementation {
    #[default]
    Stub,
}

#[derive(Copy, Clone, Debug, Display, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
#[strum(serialize_all = "kebab-case")]
enum DeviceType {
    MH160Device,
    MH160StubGenerator,
}

#[allow(clippy::too_many_lines)]
fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Info {
            mh_device_index,
            mh_wrapper_implementation,
            device_type,
        } => {
            let device = match device_type {
                DeviceType::MH160Device => match mh_wrapper_implementation {
                    #[cfg(feature = "multiharp")]
                    MhWrapperImplementation::Real => Ok::<Box<dyn MH160>, Error>(Box::new(
                        MH160Device::from_current_config(MhlibWrapperReal::new(mh_device_index))?,
                    )),
                    MhWrapperImplementation::Stub => Ok::<Box<dyn MH160>, Error>(Box::new(
                        MH160Device::from_current_config(MhlibWrapperStub::new(mh_device_index))?,
                    )),
                },
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
            mh_wrapper_implementation,
            duration,
            device_type,
            name,
        } => {
            let device = match device_type {
                DeviceType::MH160Device => {
                    match device_config {
                        Some(device_config) => {
                            let config =
                                serde_json::from_str(fs::read_to_string(&device_config)?.as_str())
                                    .with_context(|| {
                                        format!(
                                            "Error while parsing config file at {}",
                                            device_config.display()
                                        )
                                    })?;

                            match mh_wrapper_implementation {
                                #[cfg(feature = "multiharp")]
                                MhWrapperImplementation::Real => Ok::<Arc<dyn MH160>, Error>(
                                    Arc::new(MH160Device::from_config(
                                        MhlibWrapperReal::new(mh_device_index),
                                        config,
                                    )?) as Arc<(dyn MH160)>,
                                ),
                                MhWrapperImplementation::Stub => Ok::<Arc<dyn MH160>, Error>(
                                    Arc::new(MH160Device::from_config(
                                        MhlibWrapperStub::new(mh_device_index),
                                        config,
                                    )?) as Arc<(dyn MH160)>,
                                ),
                            }
                        }
                        None => {
                            match mh_wrapper_implementation {
                                #[cfg(feature = "multiharp")]
                                MhWrapperImplementation::Real => Ok::<Arc<dyn MH160>, Error>(
                                    Arc::new(MH160Device::from_current_config(
                                        MhlibWrapperReal::new(mh_device_index),
                                    )?) as Arc<(dyn MH160)>,
                                ),
                                MhWrapperImplementation::Stub => Ok::<Arc<dyn MH160>, Error>(
                                    Arc::new(MH160Device::from_current_config(
                                        MhlibWrapperStub::new(mh_device_index),
                                    )?) as Arc<(dyn MH160)>,
                                ),
                            }
                        }
                    }
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
