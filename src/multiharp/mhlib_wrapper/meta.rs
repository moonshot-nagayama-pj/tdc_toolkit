//! Definitions needed by both impl and [`stub`](super::stub).
//!
//! Many of these values are derived from `mhdefin.h`, which is bundled with the MultiHarp driver release. The values are copied here to avoid a hard dependency on downloading the proprietary MultiHarp shared library when using this library on non-x64 platforms or with non-MultiHarp systems.
//!
//! The original constant names from `mhdefin.h` are preserved as comments when they have been changed; some comments are also from `mhdefin.h`.

//limits for MH_SetRowEventFilterXXX and MH_SetMainEventFilter

/// Constant values that are used in `MH_SetRowEventFilter` and `MH_SetMainEventFilter`. Names are the same as in `mhdefin.h`.
pub mod event_filter {
    pub const ROWIDXMIN: i32 = 0;

    /// actual upper limit is smaller, depending on rows present
    pub const ROWIDXMAX: i32 = 8;

    pub const INVERSEMIN: i32 = 0;
    pub const INVERSEMAX: i32 = 1;

    /// no channels used
    pub const USECHANSMIN: i32 = 0x000;

    /// note: sync bit 0x100 will be ignored in T3 mode and in row filter
    pub const USECHANSMAX: i32 = 0x1FF;

    /// no channels passed
    pub const PASSCHANSMIN: i32 = 0x000;

    /// note: sync bit 0x100 will be ignored in T3 mode and in row filter
    pub const PASSCHANSMAX: i32 = 0x1FF;

    /// Minimum value for the matchcnt parameter; 1 means that coincidences between any 2 used channels will be recorded
    pub const MATCHCNTMIN: i32 = 1;

    /// Maximum value for the matchcnt parameter; 6 means that coincidences between any 7 used channels will be recorded
    pub const MATCHCNTMAX: i32 = 6;

    /// Minimum time range for event filters in picoseconds, e.g. the shortest possible span of time to use when doing coincidence counting.
    pub const TIMERANGEMIN: i32 = 0;

    /// Maximum time range for event filters in picoseconds, e.g. the longest possible span of time to use when doing coincidence counting.
    pub const TIMERANGEMAX: i32 = 160_000;
}

/// Number of event records that can be read by `MH_ReadFiFo`. The buffer must provide space for this number of dwords.
pub const TTREADMAX: usize = 1_048_576;

// These constants are not from `mhdefin.h`
pub const CHANNELS_PER_ROW: i32 = 8;
pub const MAX_INPUT_CHANNEL: i32 = 64;

use anyhow::Result;

#[cfg(feature = "python")]
use pyo3::prelude::*;

use serde::{Deserialize, Serialize};
use std::convert::Into;
use strum_macros::Display;

use crate::multiharp::device::MH160ChannelIdNoSync;

#[derive(Clone, Debug)]
#[repr(i32)]
#[cfg_attr(feature = "python", pyclass)]
pub enum Mode {
    Hist = 0_i32, // MODE_HIST
    T2 = 2_i32,   // MODE_T2
    T3 = 3_i32,   // MODE_T3
}

#[derive(Clone, Debug)]
#[repr(u32)]
#[cfg_attr(feature = "python", pyclass)]
pub enum RefSource {
    InternalClock = 0,                   // REFSRC_INTERNAL
    ExternalClock10MHz = 1,              // REFSRC_EXTERNAL_10MHZ
    WhiteRabbitMaster = 2,               // REFSRC_WR_MASTER_GENERIC
    WhiteRabbitSlave = 3,                // REFSRC_WR_SLAVE_GENERIC
    WhiteRabbitGrandMaster = 4,          // REFSRC_WR_GRANDM_GENERIC
    ExternalGpsPps = 5,                  // REFSRC_EXTN_GPS_PPS
    ExternalGpsPpsUart = 6,              // REFSRC_EXTN_GPS_PPS_UART
    WhiteRabbitMasterMultiHarp = 7,      // REFSRC_WR_MASTER_MHARP
    WhiteRabbitSlaveMultiHarp = 8,       // REFSRC_WR_SLAVE_MHARP
    WhiteRabbitGrandMasterMultiHarp = 9, // REFSRC_WR_GRANDM_MHARP
}

#[allow(clippy::unsafe_derive_deserialize)]
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
#[repr(i32)]
#[cfg_attr(feature = "python", pyclass)]
pub enum Edge {
    Falling = 0_i32, // EDGE_FALLING
    Rising = 1_i32,  // EDGE_RISING
}

#[derive(Clone)]
#[repr(i32)]
#[cfg_attr(feature = "python", pyclass)]
pub enum MeasurementControl {
    SingleShotCtc = 0_i32,         // MEASCTRL_SINGLESHOT_CTC
    C1Gated = 1_i32,               // MEASCTRL_C1_GATED
    C1StartCtcStop = 2_i32,        // MEASCTRL_C1_START_CTC_STOP
    C1StartC2Stop = 3_i32,         // MEASCTRL_C1_START_C2_STOP as i32
    WhiteRabbitM2S = 4_i32,        // MEASCTRL_WR_M2S
    WhiteRabbitS2M = 5_i32,        // MEASCTRL_WR_S2M
    SwitchStartSwitchStop = 6_i32, // MEASCTRL_SW_START_SW_STOP
}

/// The channel ID corresponding to the internal representation used in the official mhlib library. The ID must be greater than or equal to `0`. The sync channel cannot be represented in this scheme.
///
/// For example, the channel labeled `1` on the device's front panel is referred to as channel `0` here.
///
/// This struct is used for low-level APIs that interface directly with mhlib. Higher-level APIs use [`MH160ChannelIdNoSync`].
#[derive(PartialEq, Clone, Debug)]
pub struct MH160InternalChannelId(u8);

#[derive(Debug, Clone)]
pub struct FilteredRates {
    pub sync_rate: i32,
    pub count_rates: Vec<i32>,
}

impl MH160InternalChannelId {
    #[must_use]
    pub fn new(value: u8) -> Self {
        Self(value)
    }
}

impl From<MH160ChannelIdNoSync> for MH160InternalChannelId {
    fn from(value: MH160ChannelIdNoSync) -> Self {
        Self::new(Into::<u8>::into(value) - 1)
    }
}

impl From<MH160InternalChannelId> for u8 {
    fn from(value: MH160InternalChannelId) -> Self {
        value.0
    }
}

impl From<MH160InternalChannelId> for i32 {
    fn from(value: MH160InternalChannelId) -> Self {
        value.0.into()
    }
}

/// Used in event filtering configuration.
#[allow(clippy::unsafe_derive_deserialize)]
#[repr(i32)]
#[cfg_attr(feature = "python", pyclass)]
#[derive(Copy, Clone, Debug, Deserialize, Display, PartialEq, Serialize)]
pub enum EventFilterInverse {
    /// When the filter matches, keep the event. Discard non-matching events.
    Regular = 0,
    /// When the filter does not match, keep the event. Discard matching events.
    Inverse = 1,
}

/// Describes whether the device is in filter test mode. In test mode, no data is copied into the fifo buffer and only filtered rates are available. This is intended to allow evaluation of filter settings when data rates are too high to transfer all data.
#[allow(clippy::unsafe_derive_deserialize)]
#[repr(i32)]
#[cfg_attr(feature = "python", pyclass)]
#[derive(Copy, Clone, Debug, Deserialize, Display, PartialEq, Serialize)]
pub enum EventFilterTestMode {
    /// The device is operating normally.
    RegularOperation = 0,

    /// The device is operating in filter test mode. Data will not be available from the device.
    TestMode = 1,
}

/// Defines whether a row event filter is enabled or disabled, for the definition of "enabled" described below.
#[allow(clippy::unsafe_derive_deserialize)]
#[repr(i32)]
#[cfg_attr(feature = "python", pyclass)]
#[derive(Copy, Clone, Debug, Deserialize, Display, PartialEq, Serialize)]
pub enum RowEventFilterEnabled {
    /// When disabled, all events on this row will pass through the filter.
    Disabled = 0,
    /// When enabled, events will be filtered out if filters have been configured for that row. (The official documentation says "When it is enabled, events may be filtered out according to the parameters set with `MH_SetRowEventFilter`"; the "may be" seems to indicate that this is the behavior, but it remains untested).
    Enabled = 1,
}

/// Defines whether the main event filter is enabled or disabled, for the definition of "enabled" described below.
#[allow(clippy::unsafe_derive_deserialize)]
#[repr(i32)]
#[cfg_attr(feature = "python", pyclass)]
#[derive(Copy, Clone, Debug, Deserialize, Display, PartialEq, Serialize)]
pub enum MainEventFilterEnabled {
    /// When disabled, all events will pass through the filter.
    Disabled = 0,
    /// When enabled, events on all channels will be filtered according to the main event filter configuration, after first passing through the row event filter, if that is enabled.
    Enabled = 1,
}

pub trait MhlibWrapper: Send + Sync {
    fn clear_histogram_memory(&self) -> Result<()>;
    fn close_device(&self) -> Result<()>;
    fn ctc_status(&self) -> Result<i32>;
    fn device_index(&self) -> u8;
    fn get_all_count_rates(&self) -> Result<(i32, Vec<i32>)>;
    fn get_all_histogram(&self) -> Result<Vec<u32>>;
    fn get_base_resolution(&self) -> Result<(f64, i32)>;
    fn get_count_rate(&self, channel: MH160InternalChannelId) -> Result<i32>;
    fn get_debug_info(&self) -> Result<String>;
    fn get_elapsed_measurement_time(&self) -> Result<f64>;
    fn get_feature(&self) -> Result<i32>;
    fn get_flags(&self) -> Result<i32>;
    fn get_hardware_info(&self) -> Result<(String, String, String)>;
    fn get_histogram(&self, channel: MH160InternalChannelId) -> Result<Vec<u32>>;
    fn get_library_version(&self) -> Result<String>;
    fn get_module_info(&self, module_index: u8) -> Result<(i32, i32)>;
    fn get_number_of_input_channels(&self) -> Result<i32>;
    fn get_number_of_modules(&self) -> Result<i32>;
    fn get_resolution(&self) -> Result<f64>;
    fn get_serial_number(&self) -> Result<String>;
    fn get_start_time(&self) -> Result<(u32, u32, u32)>;
    fn get_sync_rate(&self) -> Result<i32>;
    fn get_warnings(&self) -> Result<String>;
    fn initialize(&self, mode: Mode, ref_source: RefSource) -> Result<()>;
    fn is_measurement_running(&self) -> Result<bool>;
    fn open_device(&self) -> Result<String>;
    fn read_fifo(&self) -> Result<Vec<u32>>;
    fn set_binning(&self, binning: i32) -> Result<()>;
    fn set_histogram_length(&self, len_code: i32) -> Result<i32>;
    fn set_input_channel_enable(&self, channel: MH160InternalChannelId, enable: bool)
    -> Result<()>;
    fn set_input_channel_offset(&self, channel: MH160InternalChannelId, offset: i32) -> Result<()>;
    fn set_input_deadtime(
        &self,
        channel: MH160InternalChannelId,
        on: bool,
        deadtime_ps: i32,
    ) -> Result<()>;
    fn set_input_edge_trigger(
        &self,
        channel: MH160InternalChannelId,
        level: i32,
        mac_edge: Edge,
    ) -> Result<()>;
    fn set_input_hysteresis(&self, hyst_code: u8) -> Result<()>;
    fn set_measurement_control(
        &self,
        meas_control: MeasurementControl,
        start_edge: Edge,
        stop_edge: Edge,
    ) -> Result<()>;
    fn set_offset(&self, offset: i32) -> Result<()>;
    fn set_stop_overflow(&self, stop_overflow: bool, stop_count: u32) -> Result<()>;
    fn set_sync_channel_enable(&self, enable: bool) -> Result<()>;
    fn set_sync_channel_offset(&self, sync_timing_offset: i32) -> Result<()>;
    fn set_sync_deadtime(&self, on: bool, deadtime_ps: i32) -> Result<()>;
    fn set_sync_divider(&self, divider: i32) -> Result<()>;
    fn set_sync_edge_trigger(&self, trigger_level: i32, mac_edge: Edge) -> Result<()>;
    fn set_trigger_output(&self, period_100ns: i32) -> Result<()>;
    fn start_measurement(&self, acquisition_time: i32) -> Result<()>;
    fn stop_measurement(&self) -> Result<()>;
    fn set_row_event_filter(
        &self,
        rowidx: i32,
        time_range_ps: i32,
        match_count: i32,
        inverse: EventFilterInverse,
        use_channels_bits: i32,
        pass_channels_bits: i32,
    ) -> Result<()>;

    fn enable_row_event_filter(&self, rowidx: i32, enable: RowEventFilterEnabled) -> Result<()>;

    fn set_main_event_filter_params(
        &self,
        time_range_ps: i32,
        match_count: i32,
        inverse: EventFilterInverse,
    ) -> Result<()>;

    fn set_main_event_filter_channels(
        &self,
        rowidx: i32,
        use_channels_bits: i32,
        pass_channels_bits: i32,
    ) -> Result<()>;

    fn enable_main_event_filter(&self, enable: MainEventFilterEnabled) -> Result<()>;

    fn set_filter_test_mode(&self, test_mode: EventFilterTestMode) -> Result<()>;

    fn get_row_filtered_rates(&self) -> Result<FilteredRates>;

    fn get_main_filtered_rates(&self) -> Result<FilteredRates>;
}
