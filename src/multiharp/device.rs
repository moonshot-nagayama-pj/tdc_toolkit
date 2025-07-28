//! Device implementation for real MultiHarp 160 devices.
//!
//! ## Use
//!
//! ### Lifecycle
//!
//! Generally speaking, a device is first opened, some action is
//! performed, and the device is then closed. The device is opened
//! when its struct is instantiated and closed when Drop is called.
//!
//! ### Configuration
//!
//! If the device is to be used for measurement, it must be configured
//! during initialization. The device configuration cannot be changed
//! once initialized. Instead, to change the configuration, drop the
//! device instance and create a new instance. This will close the
//! device and re-initialize it.
//!
//! The root configuration struct is [`MH160DeviceConfig`].
//!
//! Some actions, such as [`MH160Device::get_device_info()`], do not require
//! configuration.

use anyhow::{Result, anyhow, bail};
use pyo3::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fmt::{Display, Formatter};
use std::sync::mpsc;
use std::time::Duration;

use super::mhlib_wrapper;
use super::mhlib_wrapper_header::{Edge, Mode, RefSource};

/// MultiHarp 160 device configuration.
#[allow(clippy::unsafe_derive_deserialize)]
#[pyclass(get_all, set_all)]
#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub struct MH160DeviceConfig {
    /// Configuration for the sync channel. In the MultiHarp's
    /// internal representation, the sync channel is
    /// unnumbered. However, in TTTR T2 mode, the only mode
    /// tdc-toolkit currently supports, the sync channel essentially
    /// behaves as an extra channel, with no special properties.
    ///
    /// tdc-toolkit will assign the channel ID `0` to the sync channel
    /// during normalization.
    ///
    /// When this field is set to [`None`](std::option::Option::None),
    /// the sync channel is disabled.
    pub sync_channel: Option<MH160DeviceSyncChannelConfig>,

    /// Configuration for all input channels other than the sync
    /// channel. Providing a channel configuration here enables the
    /// channel; if no declaration is present for a particular
    /// channel, it is disabled.
    ///
    /// If a channel is configured twice in this vector, the last
    /// declaration takes precedence over earlier declarations.
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
    /// The channel ID, corresponding to the channel ID numbers on the
    /// MultiHarp's interface panel. The ID must be greater than or
    /// equal to `1`. `0` is reserved for the sync channel.
    ///
    /// Internally, the MultiHarp software counts channel IDs from
    /// zero and does not assign an ID to the sync channel.
    pub id: MH160ChannelId,
    pub edge_trigger_level: i32, // mV
    pub edge_trigger: Edge,
    pub offset: i32, // picoseconds
}

/// The channel ID, corresponding to the channel ID numbers on the
/// MultiHarp's interface panel. The ID must be greater than or equal
/// to `1`. `0` is reserved for the sync channel.
///
/// Internally, the MultiHarp software counts channel IDs from zero
/// and does not assign an ID to the sync channel. Lower-level APIs
/// which require that internal representation use
/// [`MH160InternalChannelId`](super::mhlib_wrapper_header::MH160InternalChannelId).
#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Copy, Clone, Debug)]
#[pyclass]
#[serde(try_from = "u8", into = "u8")]
pub struct MH160ChannelId(u8);

impl MH160ChannelId {
    pub fn new(value: u8) -> Result<Self> {
        if value > 0 {
            Ok(Self(value))
        } else {
            bail!("Value must be greater than 0, but {value} was passed.")
        }
    }
}

impl TryFrom<u8> for MH160ChannelId {
    type Error = anyhow::Error;

    fn try_from(value: u8) -> Result<Self> {
        Self::new(value)
    }
}

impl From<MH160ChannelId> for u8 {
    fn from(value: MH160ChannelId) -> Self {
        value.0
    }
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
    pub fn from_current_config(device_index: u8) -> Result<MH160Device> {
        mhlib_wrapper::open_device(device_index)?;
        mhlib_wrapper::initialize(device_index, Mode::T2, RefSource::InternalClock)?;
        Ok(MH160Device { device_index })
    }

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
            mhlib_wrapper::set_input_channel_enable(device_index, input_channel.id.into(), true)?;
            mhlib_wrapper::set_input_edge_trigger(
                device_index,
                input_channel.id.into(),
                input_channel.edge_trigger_level,
                input_channel.edge_trigger.clone(),
            )?;
            mhlib_wrapper::set_input_channel_offset(
                device_index,
                input_channel.id.into(),
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
        for channel_id in 1..=total_channels {
            #[expect(clippy::missing_panics_doc)]
            let channel_id = MH160ChannelId::new(channel_id).expect("This should not happen");
            if !enabled_channels.contains(&channel_id) {
                mhlib_wrapper::set_input_channel_enable(device_index, channel_id.into(), false)?;
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
