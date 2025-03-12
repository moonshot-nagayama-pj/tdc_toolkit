use clap::{Parser, Subcommand, ValueEnum, ValueHint};
use strum_macros::Display;
use std::fs;
use std::path::PathBuf;
use std::str::FromStr;
use std::time::Duration;

use _mhtk_rs::recording;
use _mhtk_rs::multiharp_device_stub;
use _mhtk_rs::multiharp_device;
use _mhtk_rs::multiharp_device::MultiharpDevice;

#[derive(Debug, Parser)]
#[command(name = "multiharp_toolkit")]
#[command(about = "A CLI for controlling time-to-digital converters such as the PicoQuant MultiHarp.", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    Record {
        #[arg(long, value_hint = ValueHint::DirPath, default_value_os_t = PathBuf::from_str(".").unwrap())]
        output_dir: PathBuf,
        #[arg(long, value_hint = ValueHint::FilePath, default_value_os_t = PathBuf::from_str("./conf.json").unwrap())]
        device_config: PathBuf,
        #[arg(long, default_value_t = 0)]
        device_index: u8,
        #[arg(long)]
        duration: humantime::Duration,
        #[arg(long, default_value_t = DeviceType::Multiharp160)]
        device_type: DeviceType,
        #[arg(long, default_value_t = String::from("record"))]
        name: String,
    }
}

#[derive(Copy, Clone, Debug, Display, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
#[strum(serialize_all = "kebab-case")]
enum DeviceType {
    Multiharp160,
    Multiharp160StubGenerator,
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Command::Record {
            output_dir,
            device_config,
            device_index,
            duration,
            device_type,
            name,
        } => {
            let device = match device_type {
                DeviceType::Multiharp160 => {
                    let config: multiharp_device::MultiharpDeviceConfig = serde_json::from_str(fs::read_to_string(device_config).unwrap().as_str()).unwrap();
                    Box::new(multiharp_device::Multiharp160::from_config(device_index, config)) as Box<(dyn MultiharpDevice + Send + Sync + 'static)>
                }
                DeviceType::Multiharp160StubGenerator => Box::new(multiharp_device_stub::Multiharp160Stub {}) as Box<(dyn MultiharpDevice + Send + Sync + 'static)>
            };
            recording::record_multiharp_to_parquet(device, &output_dir, Duration::from(*duration), &name);
        }
    }
}
