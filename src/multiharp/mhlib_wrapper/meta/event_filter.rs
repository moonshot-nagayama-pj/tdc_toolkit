//! Values related to event filtering. Names of simple constants are the same as in `mhdefin.h` when values are defined there.

#[cfg(feature = "python")]
use pyo3::prelude::*;

use serde::{Deserialize, Serialize};
use strum_macros::Display;

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

/// Used in event filtering configuration.
#[allow(clippy::unsafe_derive_deserialize)]
#[repr(i32)]
#[cfg_attr(feature = "python", pyclass)]
#[derive(Copy, Clone, Debug, Deserialize, Display, PartialEq, Serialize)]
pub enum Inverse {
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
pub enum TestMode {
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
pub enum RowEnabled {
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
pub enum MainEnabled {
    /// When disabled, all events will pass through the filter.
    Disabled = 0,
    /// When enabled, events on all channels will be filtered according to the main event filter configuration, after first passing through the row event filter, if that is enabled.
    Enabled = 1,
}
