use anyhow::{Result, anyhow, bail};
use pyo3::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fmt::{Display, Formatter};
use std::sync::mpsc;
use std::time::Duration;

use super::mhlib_wrapper;
use super::mhlib_wrapper_header::{Edge, Mode, RefSource};

#[allow(clippy::unsafe_derive_deserialize)]
#[pyclass(get_all, set_all)]
#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub struct MH160DeviceConfig {
    pub sync_channel: Option<MH160DeviceSyncChannelConfig>,
    pub input_channels: Vec<MH160DeviceInputChannelConfig>,
}

#[allow(clippy::unsafe_derive_deserialize)]
#[pyclass(get_all, set_all)]
#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub struct MH160DeviceSyncChannelConfig {
    pub divider: i32,
    pub edge_trigger_level: i32, // mV
    pub edge_trigger: Edge,
    pub offset: i32, // picoseconds
}

#[allow(clippy::unsafe_derive_deserialize)]
#[pyclass(get_all, set_all)]
#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub struct MH160DeviceInputChannelConfig {
    pub id: u8,
    pub edge_trigger_level: i32, // mV
    pub edge_trigger: Edge,
    pub offset: i32, // picoseconds
}

#[allow(clippy::unsafe_derive_deserialize)]
#[pyclass(get_all, str)]
#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub struct MH160DeviceInfo {
    // Amalgamation of device-related information collected from
    // several different API calls, for convenience.
    pub device_index: u8,

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

impl Display for MH160DeviceInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&serde_json::to_string(&self).unwrap())
    }
}

pub trait MH160: Send + Sync {
    fn get_device_info(&self) -> Result<MH160DeviceInfo>;
    fn stream_measurement(
        &self,
        measurement_time: &Duration,
        tx_channel: mpsc::Sender<Vec<u32>>,
    ) -> Result<()>;
}

pub struct MH160Device {
    device_index: u8,
}

impl MH160Device {
    pub fn from_config(device_index: u8, config: MH160DeviceConfig) -> Result<MH160Device> {
        mhlib_wrapper::open_device(device_index)?;

        // TODO in theory we could support T3 mode relatively easily,
        // since the record processing is decoupled from the device
        mhlib_wrapper::initialize(device_index, Mode::T2, RefSource::InternalClock)?;

        // TODO sync channel must be enabled for histogramming and T3
        // mode; more configuration validation is necessary if we want
        // to support those modes. Conversely, we don't need to use
        // sync in T2; in theory we could just permanently disable it.
        match config.sync_channel {
            Some(sync_config) => {
                mhlib_wrapper::set_sync_channel_enable(device_index, true)?;
                mhlib_wrapper::set_sync_divider(device_index, sync_config.divider)?;
                mhlib_wrapper::set_sync_edge_trigger(
                    device_index,
                    sync_config.edge_trigger_level,
                    sync_config.edge_trigger,
                )?;
                mhlib_wrapper::set_sync_channel_offset(device_index, sync_config.offset)?;
            }
            None => {
                mhlib_wrapper::set_sync_channel_enable(device_index, false)?;
            }
        }

        for input_channel in &config.input_channels {
            mhlib_wrapper::set_input_channel_enable(device_index, input_channel.id, true)?;
            mhlib_wrapper::set_input_edge_trigger(
                device_index,
                input_channel.id,
                input_channel.edge_trigger_level,
                input_channel.edge_trigger.clone(),
            )?;
            mhlib_wrapper::set_input_channel_offset(
                device_index,
                input_channel.id,
                input_channel.offset,
            )?;
        }

        // disable all other input channels
        let enabled_channels = config
            .input_channels
            .iter()
            .fold(HashSet::new(), |mut acc, x| {
                acc.insert(x.id);
                acc
            });
        let total_channels: u8 =
            mhlib_wrapper::get_number_of_input_channels(device_index)?.try_into()?;
        for channel_id in 0..total_channels {
            if !enabled_channels.contains(&channel_id) {
                mhlib_wrapper::set_input_channel_enable(device_index, channel_id, false)?;
            }
        }

        Ok(MH160Device { device_index })
    }

    fn do_stream_measurement(
        &self,
        measurement_time: &Duration,
        tx_channel: &mpsc::Sender<Vec<u32>>,
    ) -> Result<()> {
        mhlib_wrapper::start_measurement(
            self.device_index,
            measurement_time.as_millis().try_into()?,
        )?;
        loop {
            let flags = mhlib_wrapper::get_flags(self.device_index)?;
            if flags & 2 > 0 {
                // FLAG_FIFOFULL
                bail!("FLAG_FIFOFULL seen, FIFO overrun. Stopping measurement.");
            }
            let records = mhlib_wrapper::read_fifo(self.device_index)?;
            if !records.is_empty() {
                tx_channel.send(records)?;
            } else if mhlib_wrapper::ctc_status(self.device_index)? != 0 {
                // measurement completed
                break;
            }
        }
        // measurement is stopped in higher-level function
        Ok(())
    }
}

impl MH160 for MH160Device {
    fn get_device_info(&self) -> Result<MH160DeviceInfo> {
        let (model, partno, version) = mhlib_wrapper::get_hardware_info(self.device_index)?;
        let (base_resolution, binsteps) = mhlib_wrapper::get_base_resolution(self.device_index)?;
        Ok(MH160DeviceInfo {
            device_index: self.device_index,
            library_version: mhlib_wrapper::get_library_version()?,
            model,
            partno,
            version,
            serial_number: mhlib_wrapper::get_serial_number(self.device_index)?,
            base_resolution,
            binsteps: binsteps.try_into()?,
            num_channels: mhlib_wrapper::get_number_of_input_channels(self.device_index)?
                .try_into()?,
        })
    }

    fn stream_measurement(
        &self,
        measurement_time: &Duration,
        tx_channel: mpsc::Sender<Vec<u32>>,
    ) -> Result<()> {
        let measurement_result = self.do_stream_measurement(measurement_time, &tx_channel);
        let stop_result = mhlib_wrapper::stop_measurement(self.device_index);
        if let Err(root_error) = measurement_result {
            let mut final_error =
                anyhow!("Error while performing MultiHarp measurement.").context(root_error);
            if let Err(stop_error) = stop_result {
                final_error = final_error.context(stop_error);
            }
            return Err(final_error);
        }
        Ok(())
    }
}

impl Drop for MH160Device {
    fn drop(&mut self) {
        if let Err(e) = mhlib_wrapper::close_device(self.device_index) {
            panic!("Error while closing MultiHarp. {e:?}");
        }
    }
}
