use pyo3::prelude::*;
use serde::{Deserialize, Serialize};

// These values are derived from mhdefin.h, which is bundled with the
// MultiHarp driver release. The values are copied here to avoid a
// hard dependency on the using mhlib module or downloading the
// proprietary MultiHarp shared library when using this library on
// non-x64 platforms or with non-MultiHarp systems.
//
// The original constant names are preserved as comments.

pub const TTREADMAX: usize = 1048576;

#[derive(Clone, Debug)]
#[repr(i32)]
#[pyclass]
pub enum Mode {
    Hist = 0_i32, // MODE_HIST
    T2 = 2_i32,   // MODE_T2
    T3 = 3_i32,   // MODE_T3
}

#[derive(Clone, Debug)]
#[repr(u32)]
#[pyclass]
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

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
#[repr(i32)]
#[pyclass]
pub enum Edge {
    Falling = 0_i32, // EDGE_FALLING
    Rising = 1_i32,  // EDGE_RISING
}

#[derive(Clone)]
#[repr(i32)]
#[pyclass]
pub enum MeasurementControl {
    SingleShotCtc = 0_i32,         // MEASCTRL_SINGLESHOT_CTC
    C1Gated = 1_i32,               // MEASCTRL_C1_GATED
    C1StartCtcStop = 2_i32,        // MEASCTRL_C1_START_CTC_STOP
    C1StartC2Stop = 3_i32,         // MEASCTRL_C1_START_C2_STOP as i32
    WhiteRabbitM2S = 4_i32,        // MEASCTRL_WR_M2S
    WhiteRabbitS2M = 5_i32,        // MEASCTRL_WR_S2M
    SwitchStartSwitchStop = 6_i32, // MEASCTRL_SW_START_SW_STOP
}
