pub mod mhlib_wrapper_header;
use mhlib_wrapper_header::{Edge, MeasurementControl, Mode, RefSource};

#[cfg_attr(
    any(
        not(any(
            all(target_arch = "x86_64", target_os = "windows"),
            all(target_arch = "x86_64", target_os = "linux")
        )),
        feature = "stub"
    ),
    path = "stub_mhlib_wrapper.rs"
)]
pub mod mhlib_wrapper;

pub mod multiharp_device;
pub mod multiharp_device_stub;
pub mod parquet_writer;
pub mod recording;
pub mod tttr_record;
