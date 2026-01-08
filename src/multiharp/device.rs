//! Device implementation for real MultiHarp 160 devices.
//!
//! ## Use
//!
//! ### Lifecycle
//!
//! Generally speaking, a device is first opened, some action is performed, and the device is then closed. The device is opened when its struct is instantiated and closed when Drop is called.
//!
//! ### Configuration
//!
//! If the device is to be used for measurement, it must be configured during initialization. The device configuration cannot be changed once initialized. Instead, to change the configuration, drop the device instance and create a new instance. This will close the device and re-initialize it.
//!
//! The root configuration struct is [`MH160DeviceConfig`].
//!
//! Some actions, such as [`MH160Device::device_info()`], do not require configuration.

use anyhow::{Result, anyhow, bail, ensure};

#[cfg(feature = "python")]
use pyo3::pyclass;

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fmt::{Display, Formatter};
use std::sync::mpsc;
use std::time::Duration;

use super::mhlib_wrapper::meta::{
    CHANNELS_PER_ROW, Edge, EventFilterInverse, EventFilterTestMode, MainEventFilterEnabled,
    MhlibWrapper, Mode, RefSource, RowEventFilterEnabled,
};

/// MultiHarp 160 device configuration.
#[allow(clippy::unsafe_derive_deserialize)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub struct MH160DeviceConfig {
    /// Configuration for the sync channel. In the MultiHarp's internal representation, the sync channel is unnumbered. However, in TTTR T2 mode, the only mode `tdc_toolkit` currently supports, the sync channel essentially behaves as an extra channel, with no special properties.
    ///
    /// `tdc_toolkit` will assign the channel ID `0` to the sync channel during normalization.
    ///
    /// When this field is set to [`None`], the sync channel is disabled.
    pub sync_channel: Option<MH160DeviceSyncChannelConfig>,

    /// Configuration for all input channels other than the sync channel. Providing a channel configuration here enables the channel; if no declaration is present for a particular channel, it is disabled.
    ///
    /// Attempting to configure the same channel more than once will cause an error.
    pub input_channels: MH160DeviceInputChannelConfigs,
    pub main_event_filter: Option<MainEventFilterConfig>,
    pub row_event_filters: Option<Vec<RowEventFilterConfig>>,
}

#[allow(clippy::unsafe_derive_deserialize)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub struct MH160DeviceSyncChannelConfig {
    pub divider: i32,
    pub edge_trigger_level: i32, // mV
    pub edge_trigger: Edge,
    pub offset: i32, // picoseconds
}

#[allow(clippy::unsafe_derive_deserialize)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub struct MH160DeviceInputChannelConfig {
    /// The channel ID, corresponding to the channel ID numbers on the MultiHarp's interface panel. The ID must be greater than or equal to `1`. `0` is reserved for the sync channel.
    ///
    /// Internally, the MultiHarp software counts channel IDs from zero and does not assign an ID to the sync channel.
    pub id: MH160ChannelIdNoSync,
    pub edge_trigger_level: i32, // mV
    pub edge_trigger: Edge,
    pub offset: i32, // picoseconds
}

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
#[cfg_attr(feature = "python", pyclass)]
#[serde(
    try_from = "Vec<MH160DeviceInputChannelConfig>",
    into = "Vec<MH160DeviceInputChannelConfig>"
)]
pub struct MH160DeviceInputChannelConfigs {
    pub channels: Vec<MH160DeviceInputChannelConfig>,
}

impl MH160DeviceInputChannelConfigs {
    pub fn try_new(channels: Vec<MH160DeviceInputChannelConfig>) -> Result<Self> {
        Self::check_duplicates(&channels)?;
        Ok(Self { channels })
    }

    fn check_duplicates(configs: &Vec<MH160DeviceInputChannelConfig>) -> Result<()> {
        let mut id_counts = HashMap::new();
        for config in configs {
            id_counts
                .entry(config.id)
                .and_modify(|count| *count += 1)
                .or_insert(1);
        }
        let duplicate_ids: Vec<_> = id_counts.extract_if(|_k, v| *v > 1).collect();
        if !duplicate_ids.is_empty() {
            bail!(
                "More than one configuration for the following channel IDs was found: {duplicate_ids:#?}"
            )
        }
        Ok(())
    }
}

impl TryFrom<Vec<MH160DeviceInputChannelConfig>> for MH160DeviceInputChannelConfigs {
    type Error = anyhow::Error;

    fn try_from(value: Vec<MH160DeviceInputChannelConfig>) -> Result<Self> {
        Self::try_new(value)
    }
}

impl From<MH160DeviceInputChannelConfigs> for Vec<MH160DeviceInputChannelConfig> {
    fn from(value: MH160DeviceInputChannelConfigs) -> Self {
        value.channels
    }
}

/// The channel ID, corresponding to the channel ID numbers on the MultiHarp's interface panel. The ID must be greater than or equal to `1`. `0` is reserved for the sync channel.
///
/// Internally, the MultiHarp software counts channel IDs from zero and does not assign an ID to the sync channel. Lower-level APIs which require that internal representation use [`MH160InternalChannelId`](super::mhlib_wrapper::meta::MH160InternalChannelId).
#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Copy, Clone, Debug)]
#[cfg_attr(feature = "python", pyclass)]
#[serde(try_from = "u8", into = "u8")]
pub struct MH160ChannelIdNoSync(u8);

impl MH160ChannelIdNoSync {
    pub fn new(value: u8) -> Result<Self> {
        if value > 0 {
            Ok(Self(value))
        } else {
            bail!("Value must be greater than 0, but {value} was passed.")
        }
    }
}

impl TryFrom<u8> for MH160ChannelIdNoSync {
    type Error = anyhow::Error;

    fn try_from(value: u8) -> Result<Self> {
        Self::new(value)
    }
}

impl From<MH160ChannelIdNoSync> for u8 {
    fn from(value: MH160ChannelIdNoSync) -> Self {
        value.0
    }
}

/// When configuring event filters, the sync channel is treated like an ordinary channel. When using the sync channel as an ordinary channel, we usually refer to it as 0.
///
/// Even though the mhlib library expects the sync channel to be called channel 8 here (in a row of eight channels numbered from 0 to 7, the sync channel is 8), we continue to refer to it as channel 0 here, because in a multi-row configuration counting from 0, channel 8 would refer to the first channel of the second row.
///
/// Do not confuse this type with `MH160InternalChannelId`, which is also 0-based, but can only represent ordinary channels, not the sync channel.
#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Copy, Clone, Debug)]
#[cfg_attr(feature = "python", pyclass)]
#[serde(try_from = "u8", into = "u8")]
pub struct MH160ChannelIdZeroIsSync(u8);

impl MH160ChannelIdZeroIsSync {
    const SYNC: Self = Self::new(0);

    #[must_use]
    pub const fn new(value: u8) -> Self {
        Self(value)
    }
}

impl From<u8> for MH160ChannelIdZeroIsSync {
    fn from(value: u8) -> Self {
        Self::new(value)
    }
}

impl From<MH160ChannelIdZeroIsSync> for u8 {
    fn from(value: MH160ChannelIdZeroIsSync) -> Self {
        value.0
    }
}

#[allow(clippy::unsafe_derive_deserialize)]
#[cfg_attr(feature = "python", pyclass(get_all, str))]
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
    pub num_channels: u16,

    // Derived value
    pub num_rows: u16,
}

impl Display for MH160DeviceInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&serde_json::to_string(&self).unwrap())
    }
}

pub trait MH160: Send + Sync {
    fn device_info(&self) -> MH160DeviceInfo;
    fn stream_measurement(
        &self,
        measurement_time: &Duration,
        tx_channel: mpsc::Sender<Vec<u32>>,
    ) -> Result<()>;
}

pub struct MH160Device<T: MhlibWrapper> {
    device_info: MH160DeviceInfo,
    mhlib_wrapper: T,
}

#[allow(clippy::unsafe_derive_deserialize)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct RowEventFilterConfig {
    pub time_range_ps: i32,
    pub inverse: EventFilterInverse,
    #[serde(default)]
    pub pass_channels: Vec<MH160ChannelIdNoSync>,
    pub use_channels: Vec<MH160ChannelIdNoSync>,
    pub match_count: i32,
}

#[allow(clippy::unsafe_derive_deserialize)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct MainEventFilterConfig {
    pub time_range_ps: i32,
    pub inverse: EventFilterInverse,
    pub match_count: i32,
    #[serde(default)]
    pub pass_channels: Vec<MH160ChannelIdZeroIsSync>,
    pub use_channels: Vec<MH160ChannelIdZeroIsSync>,
}

impl<T: MhlibWrapper> MH160Device<T> {
    pub fn from_current_config(mhlib_wrapper: T) -> Result<MH160Device<T>> {
        mhlib_wrapper.open_device()?;
        mhlib_wrapper.initialize(Mode::T2, RefSource::InternalClock)?;
        let device_info = Self::get_device_info(&mhlib_wrapper)?;
        Ok(MH160Device {
            device_info,
            mhlib_wrapper,
        })
    }

    #[allow(clippy::too_many_lines)]
    pub fn from_config(mhlib_wrapper: T, config: &MH160DeviceConfig) -> Result<MH160Device<T>> {
        mhlib_wrapper.open_device()?;

        // TODO in theory we could support T3 mode relatively easily,
        // since the record processing is decoupled from the device
        mhlib_wrapper.initialize(Mode::T2, RefSource::InternalClock)?;

        let device_info = Self::get_device_info(&mhlib_wrapper)?;

        // TODO sync channel must be enabled for histogramming and T3
        // mode; more configuration validation is necessary if we want
        // to support those modes. Conversely, we don't need to use
        // sync in T2; in theory we could just permanently disable it.
        match &config.sync_channel {
            Some(sync_config) => {
                mhlib_wrapper.set_sync_channel_enable(true)?;
                mhlib_wrapper.set_sync_divider(sync_config.divider)?;
                mhlib_wrapper.set_sync_edge_trigger(
                    sync_config.edge_trigger_level,
                    sync_config.edge_trigger.clone(),
                )?;
                mhlib_wrapper.set_sync_channel_offset(sync_config.offset)?;
            }
            None => {
                mhlib_wrapper.set_sync_channel_enable(false)?;
            }
        }

        let input_channels = &config.input_channels.channels;
        for input_channel in input_channels {
            mhlib_wrapper.set_input_channel_enable(input_channel.id.into(), true)?;
            mhlib_wrapper.set_input_edge_trigger(
                input_channel.id.into(),
                input_channel.edge_trigger_level,
                input_channel.edge_trigger.clone(),
            )?;
            mhlib_wrapper
                .set_input_channel_offset(input_channel.id.into(), input_channel.offset)?;
        }

        // disable all other input channels
        let enabled_channels = input_channels.iter().fold(HashSet::new(), |mut acc, x| {
            acc.insert(x.id);
            acc
        });
        let total_channels: u8 = mhlib_wrapper.get_number_of_input_channels()?.try_into()?;
        for channel_id in 1..=total_channels {
            #[expect(clippy::missing_panics_doc)]
            let channel_id = MH160ChannelIdNoSync::new(channel_id).expect("This should not happen");
            if !enabled_channels.contains(&channel_id) {
                mhlib_wrapper.set_input_channel_enable(channel_id.into(), false)?;
            }
        }

        mhlib_wrapper.set_filter_test_mode(EventFilterTestMode::RegularOperation)?;
        Self::configure_row_filters(&mhlib_wrapper, config, &device_info)?;
        Self::configure_main_filter(&mhlib_wrapper, config, &device_info)?;

        Ok(MH160Device {
            device_info,
            mhlib_wrapper,
        })
    }

    fn do_stream_measurement(
        &self,
        measurement_time: &Duration,
        tx_channel: &mpsc::Sender<Vec<u32>>,
    ) -> Result<()> {
        self.mhlib_wrapper
            .start_measurement(measurement_time.as_millis().try_into()?)?;
        loop {
            let flags = self.mhlib_wrapper.get_flags()?;
            if flags & 2 > 0 {
                // FLAG_FIFOFULL
                bail!("FLAG_FIFOFULL seen, FIFO overrun. Stopping measurement.");
            }
            let records = self.mhlib_wrapper.read_fifo()?;
            if !records.is_empty() {
                tx_channel.send(records)?;
            } else if self.mhlib_wrapper.ctc_status()? != 0 {
                // measurement completed
                break;
            }
        }
        // measurement is stopped in higher-level function
        Ok(())
    }

    fn get_device_info(mhlib_wrapper: &T) -> Result<MH160DeviceInfo> {
        let (model, partno, version) = mhlib_wrapper.get_hardware_info()?;
        let (base_resolution, binsteps) = mhlib_wrapper.get_base_resolution()?;

        let num_channels: u16 = mhlib_wrapper.get_number_of_input_channels()?.try_into()?;

        let channels_per_row = u16::try_from(CHANNELS_PER_ROW)?;
        anyhow::ensure!(
            num_channels.is_multiple_of(channels_per_row),
            "input channels ({}) is not divisible by the number of channels in each row ({}), this does not make sense",
            num_channels,
            CHANNELS_PER_ROW
        );
        let num_rows = num_channels / channels_per_row;

        Ok(MH160DeviceInfo {
            device_index: mhlib_wrapper.device_index(),
            library_version: mhlib_wrapper.get_library_version()?,
            model,
            partno,
            version,
            serial_number: mhlib_wrapper.get_serial_number()?,
            base_resolution,
            binsteps: binsteps.try_into()?,
            num_channels,
            num_rows,
        })
    }

    fn configure_row_filters(
        mhlib_wrapper: &T,
        config: &MH160DeviceConfig,
        device_info: &MH160DeviceInfo,
    ) -> Result<()> {
        match &config.row_event_filters {
            Some(row_filters) => Self::enable_row_filters(mhlib_wrapper, row_filters, device_info),
            None => Self::disable_row_filters(mhlib_wrapper, device_info),
        }
    }

    fn enable_row_filters(
        mhlib_wrapper: &T,
        row_filters: &[RowEventFilterConfig],
        device_info: &MH160DeviceInfo,
    ) -> Result<()> {
        let num_rows_usize = usize::from(device_info.num_rows);
        let num_row_filters = row_filters.len();
        ensure!(
            num_row_filters <= num_rows_usize,
            "Attempted to configure {num_row_filters} row event filters, but device has only {num_rows_usize} rows.",
        );

        for rowidx_usize in 0..num_rows_usize {
            let rowidx_i32: i32 = rowidx_usize.try_into()?;

            match row_filters.get(rowidx_usize) {
                Some(rf) => {
                    let use_bits: i32 = row_filter_row_mask(&rf.use_channels, rowidx_i32);
                    let pass_bits: i32 = row_filter_row_mask(&rf.pass_channels, rowidx_i32);
                    mhlib_wrapper.set_row_event_filter(
                        rowidx_i32,
                        rf.time_range_ps,
                        rf.match_count,
                        rf.inverse,
                        use_bits,
                        pass_bits,
                    )?;
                    mhlib_wrapper
                        .enable_row_event_filter(rowidx_i32, RowEventFilterEnabled::Enabled)?;
                }
                None => mhlib_wrapper
                    .enable_row_event_filter(rowidx_i32, RowEventFilterEnabled::Disabled)?, // TODO should we enable it and mask all rows to block pass-through, or make this more configurable?
            }
        }
        Ok(())
    }

    fn disable_row_filters(mhlib_wrapper: &T, device_info: &MH160DeviceInfo) -> Result<()> {
        for rowidx in 0..device_info.num_rows {
            mhlib_wrapper
                .enable_row_event_filter(i32::from(rowidx), RowEventFilterEnabled::Disabled)?;
        }
        Ok(())
    }

    fn configure_main_filter(
        mhlib_wrapper: &T,
        config: &MH160DeviceConfig,
        device_info: &MH160DeviceInfo,
    ) -> Result<()> {
        match &config.main_event_filter {
            Some(main_filter) => Self::enable_main_filter(mhlib_wrapper, main_filter, device_info),
            None => mhlib_wrapper.enable_main_event_filter(MainEventFilterEnabled::Disabled),
        }
    }

    fn enable_main_filter(
        mhlib_wrapper: &T,
        main: &MainEventFilterConfig,
        device_info: &MH160DeviceInfo,
    ) -> Result<()> {
        mhlib_wrapper.enable_main_event_filter(MainEventFilterEnabled::Enabled)?;

        mhlib_wrapper.set_main_event_filter_params(
            main.time_range_ps,
            main.match_count,
            main.inverse,
        )?;

        let num_rows_i32: i32 = device_info.num_rows.into();
        for rowidx in 0..num_rows_i32 {
            let use_bits = main_filter_row_mask(&main.use_channels, rowidx);
            let pass_bits = main_filter_row_mask(&main.pass_channels, rowidx);
            mhlib_wrapper.set_main_event_filter_channels(rowidx, use_bits, pass_bits)?;
        }
        Ok(())
    }
}

impl<T: MhlibWrapper> MH160 for MH160Device<T> {
    fn device_info(&self) -> MH160DeviceInfo {
        self.device_info.clone()
    }

    fn stream_measurement(
        &self,
        measurement_time: &Duration,
        tx_channel: mpsc::Sender<Vec<u32>>,
    ) -> Result<()> {
        let measurement_result = self.do_stream_measurement(measurement_time, &tx_channel);
        let stop_result = self.mhlib_wrapper.stop_measurement();
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

impl<T: MhlibWrapper> Drop for MH160Device<T> {
    fn drop(&mut self) {
        for rowidx in 0..self.device_info.num_rows {
            // TODO need to unset filters here and also consolidate the two loops into one
            let _ = self
                .mhlib_wrapper
                .enable_row_event_filter(rowidx.into(), RowEventFilterEnabled::Disabled);
        }
        for rowidx in 0..self.device_info.num_rows {
            let _ = self
                .mhlib_wrapper
                .set_main_event_filter_channels(rowidx.into(), 0, 0);
        }
        let _ = self
            .mhlib_wrapper
            .enable_main_event_filter(MainEventFilterEnabled::Disabled);
        let _ = self
            .mhlib_wrapper
            .set_filter_test_mode(EventFilterTestMode::RegularOperation);
        if let Err(e) = self.mhlib_wrapper.close_device() {
            eprintln!("Warning: error while closing MultiHarp: {e:?}");
        }
    }
}

/// The row filters cannot include the sync channel (or, at least, the official documentation doesn't indicate that they can), so the bitmasking logic is slightly different from that used for the main filter.
fn row_filter_row_mask(global_1_channels: &[MH160ChannelIdNoSync], rowidx: i32) -> i32 {
    let mut bits = 0u16;
    let global_row_start = rowidx * CHANNELS_PER_ROW;
    let global_row_end = global_row_start + CHANNELS_PER_ROW - 1;
    for global_1_channel in global_1_channels {
        let global_1_channel_zero_is_ordinary = i32::from(global_1_channel.0 - 1);
        if global_1_channel_zero_is_ordinary >= global_row_start
            && global_1_channel_zero_is_ordinary <= global_row_end
        {
            let local_1_channel = global_1_channel_zero_is_ordinary - global_row_start;
            bits |= 1u16 << local_1_channel;
        }
    }
    i32::from(bits)
}

/// The main filter can include the sync channel, when the sync channel is being used as an ordinary channel in T2 mode. However, handling the sync channel requires a slightly different bitmask encoding than is used for the row filter.
fn main_filter_row_mask(global_1_channels: &[MH160ChannelIdZeroIsSync], rowidx: i32) -> i32 {
    let mut bits = 0u16;
    if rowidx == 0 && global_1_channels.contains(&MH160ChannelIdZeroIsSync::SYNC) {
        bits |= 1u16 << 8;
    }

    let global_row_start = rowidx * CHANNELS_PER_ROW;
    let global_row_end = global_row_start + CHANNELS_PER_ROW - 1;
    for global_1_channel in global_1_channels {
        if global_1_channel == &MH160ChannelIdZeroIsSync::SYNC {
            continue;
        }
        let global_1_channel_zero_is_ordinary = i32::from(global_1_channel.0 - 1);
        if global_1_channel_zero_is_ordinary >= global_row_start
            && global_1_channel_zero_is_ordinary <= global_row_end
        {
            let local_1_channel = global_1_channel_zero_is_ordinary - global_row_start;
            bits |= 1u16 << local_1_channel;
        }
    }
    i32::from(bits)
}

// TODO need to write unit tests for masking
