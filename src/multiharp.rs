pub mod mhlib_wrapper_header;

#[cfg_attr(
    any(
        not(any(
            all(target_arch = "x86_64", target_os = "windows"),
            all(target_arch = "x86_64", target_os = "linux")
        )),
        not(feature = "multiharp")
    ),
    path = "mhlib_wrapper_stub.rs"
)]
pub mod mhlib_wrapper;

pub mod device;
pub mod device_stub;
pub mod recording;
pub mod tttr_record;
