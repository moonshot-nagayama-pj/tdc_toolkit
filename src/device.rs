use std::collections::HashSet;
use std::sync::mpsc;
use std::time::Duration;

use crate::mhlib_wrapper;
use crate::mhlib_wrapper_enums::{Edge, Mode, RefSource};

#[derive(PartialEq, Clone, Debug)]
pub struct MultiharpDeviceConfig {
    sync_channel: Option<MultiharpDeviceSyncChannelConfig>,
    input_channels: Vec<MultiharpDeviceInputChannelConfig>,
}

#[derive(PartialEq, Clone, Debug)]
pub struct MultiharpDeviceSyncChannelConfig {
    divider: i32,
    edge_trigger_level: i32, // mV
    edge_trigger: Edge,
    offset: i32, // picoseconds
}

#[derive(PartialEq, Clone, Debug)]
pub struct MultiharpDeviceInputChannelConfig {
    id: u8,
    edge_trigger_level: i32, // mV
    edge_trigger: Edge,
    offset: i32, // picoseconds
}

#[derive(PartialEq, Clone, Debug)]
pub struct MultiharpDeviceInfo {
    // Amalgamation of device-related information collected from
    // several different API calls, for convenience.
    pub device_index: u32,

    // MH_GetLibraryVersion
    pub library_version: String,

    // MH_GetHardwareInfo
    pub model: String,
    pub partno: String,
    pub version: String,

    // MH_GetSerialNumber
    pub serial_number: String,

    // MH_GetBaseResolution
    pub base_resolution: f64,
    pub binsteps: u32,

    // MH_GetNumOfInputChannels
    pub num_channels: u32,
}

pub struct MultiharpDevice {
    device_index: u8,
}

impl MultiharpDevice {
    pub fn from_config(device_index: u8, config: MultiharpDeviceConfig) -> MultiharpDevice {
        mhlib_wrapper::open_device(device_index).unwrap();

        // TODO in theory we could support T3 mode relatively easily,
        // since the record processing is decoupled from the device
        mhlib_wrapper::initialize(device_index, Mode::T2, RefSource::InternalClock).unwrap();

        // TODO sync channel must be enabled for histogramming and T3
        // mode; more configuration validation is necessary if we want
        // to support those modes. Conversely, we don't need to use
        // sync in T2; in theory we could just permanently disable it.
        match config.sync_channel {
            Some(sync_config) => {
                mhlib_wrapper::set_sync_channel_enable(device_index, true).unwrap();
                mhlib_wrapper::set_sync_divider(device_index, sync_config.divider).unwrap();
                mhlib_wrapper::set_sync_edge_trigger(
                    device_index,
                    sync_config.edge_trigger_level,
                    sync_config.edge_trigger,
                )
                .unwrap();
                mhlib_wrapper::set_sync_channel_offset(device_index, sync_config.offset).unwrap();
            }
            None => {
                mhlib_wrapper::set_sync_channel_enable(device_index, false).unwrap();
            }
        }

        for input_channel in config.input_channels.iter() {
            mhlib_wrapper::set_input_channel_enable(device_index, input_channel.id, true).unwrap();
            mhlib_wrapper::set_input_edge_trigger(
                device_index,
                input_channel.id,
                input_channel.edge_trigger_level,
                input_channel.edge_trigger.clone(),
            )
            .unwrap();
            mhlib_wrapper::set_input_channel_offset(
                device_index,
                input_channel.id,
                input_channel.offset,
            )
            .unwrap();
        }

        // disable all other input channels
        let enabled_channels = config
            .input_channels
            .iter()
            .fold(HashSet::new(), |mut acc, x| {
                acc.insert(x.id);
                acc
            });
        let total_channels: u8 = mhlib_wrapper::get_number_of_input_channels(device_index)
            .unwrap()
            .try_into()
            .unwrap();
        for channel_id in 0..=total_channels {
            if !enabled_channels.contains(&channel_id) {
                mhlib_wrapper::set_input_channel_enable(device_index, channel_id, false).unwrap();
            }
        }

        MultiharpDevice { device_index }
    }

    pub fn get_device_info(&self) -> MultiharpDeviceInfo {
        let (model, partno, version) = mhlib_wrapper::get_hardware_info(self.device_index).unwrap();
        let (base_resolution, binsteps) =
            mhlib_wrapper::get_base_resolution(self.device_index).unwrap();
        MultiharpDeviceInfo {
            device_index: 1,
            library_version: mhlib_wrapper::get_library_version().unwrap(),
            model,
            partno,
            version,
            serial_number: mhlib_wrapper::get_serial_number(self.device_index).unwrap(),
            base_resolution,
            binsteps: binsteps.try_into().unwrap(),
            num_channels: mhlib_wrapper::get_number_of_input_channels(self.device_index)
                .unwrap()
                .try_into()
                .unwrap(),
        }
    }

    pub fn stream_measurement(
        &self,
        measurement_time: &Duration,
        tx_channel: mpsc::Sender<Vec<u32>>,
    ) {
        mhlib_wrapper::start_measurement(
            self.device_index,
            measurement_time.as_millis().try_into().unwrap(),
        )
        .unwrap();
        loop {
            let flags = mhlib_wrapper::get_flags(self.device_index).unwrap();
            if flags & 2 > 0 {
                // FLAG_FIFOFULL
                panic!("FIFO overrun");
            }
            let records = mhlib_wrapper::read_fifo_to_vec(self.device_index).unwrap();
            if !records.is_empty() {
                tx_channel.send(records).unwrap();
            } else if mhlib_wrapper::ctc_status(self.device_index).unwrap() != 0 {
                // measurement completed
                break;
            }
        }
        mhlib_wrapper::stop_measurement(self.device_index).unwrap();
        // TODO how to implement the equivalent of try/finally? Put
        // this in its own higher-level function? Make a measurement
        // into into its own struct, implementing Drop?
    }
}

impl Drop for MultiharpDevice {
    fn drop(&mut self) {
        mhlib_wrapper::close_device(self.device_index).unwrap();
    }
}
