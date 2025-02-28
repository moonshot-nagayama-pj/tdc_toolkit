use pyo3::prelude::*;

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

pub mod device;
pub mod parquet_writer;
pub mod stub_device;
pub mod tttr_record;

mod mhlib_wrapper_py;
use mhlib_wrapper_py::*;

mod parser;
mod parser_py;
use parser::PtuParser;

#[pymodule]
#[pyo3(name = "_mhtk_rs")]
fn _mhtk_rs(_py: Python, m: &PyModule) -> PyResult<()> {
    {
        m.add_function(wrap_pyfunction!(get_library_version, m)?)?;
        m.add_function(wrap_pyfunction!(open_device, m)?)?;
        m.add_function(wrap_pyfunction!(close_device, m)?)?;
        m.add_function(wrap_pyfunction!(initialize, m)?)?;
        m.add_function(wrap_pyfunction!(get_hardware_info, m)?)?;
        m.add_function(wrap_pyfunction!(get_feature, m)?)?;
        m.add_function(wrap_pyfunction!(get_serial_number, m)?)?;
        m.add_function(wrap_pyfunction!(get_base_resolution, m)?)?;
        m.add_function(wrap_pyfunction!(get_number_of_input_channels, m)?)?;
        m.add_function(wrap_pyfunction!(get_number_of_modules, m)?)?;
        m.add_function(wrap_pyfunction!(get_module_info, m)?)?;
        m.add_function(wrap_pyfunction!(get_debug_info, m)?)?;
        m.add_function(wrap_pyfunction!(set_sync_divider, m)?)?;
        m.add_function(wrap_pyfunction!(set_sync_edge_trigger, m)?)?;
        m.add_function(wrap_pyfunction!(set_sync_channel_offset, m)?)?;
        m.add_function(wrap_pyfunction!(set_sync_channel_enable, m)?)?;
        m.add_function(wrap_pyfunction!(set_sync_deadtime, m)?)?;
        m.add_function(wrap_pyfunction!(set_input_edge_trigger, m)?)?;
        m.add_function(wrap_pyfunction!(set_input_channel_offset, m)?)?;
        m.add_function(wrap_pyfunction!(set_input_channel_enable, m)?)?;
        m.add_function(wrap_pyfunction!(set_input_deadtime, m)?)?;
        m.add_function(wrap_pyfunction!(set_input_hysteresis, m)?)?;
        m.add_function(wrap_pyfunction!(set_stop_overflow, m)?)?;
        m.add_function(wrap_pyfunction!(set_binning, m)?)?;
        m.add_function(wrap_pyfunction!(set_offset, m)?)?;
        m.add_function(wrap_pyfunction!(set_histogram_length, m)?)?;
        m.add_function(wrap_pyfunction!(clear_histogram_memory, m)?)?;
        m.add_function(wrap_pyfunction!(set_measurement_control, m)?)?;
        m.add_function(wrap_pyfunction!(set_trigger_output, m)?)?;
        m.add_function(wrap_pyfunction!(start_measurement, m)?)?;
        m.add_function(wrap_pyfunction!(stop_measurement, m)?)?;
        m.add_function(wrap_pyfunction!(ctc_status, m)?)?;
        m.add_function(wrap_pyfunction!(get_histogram, m)?)?;
        m.add_function(wrap_pyfunction!(get_all_histograms, m)?)?;
        m.add_function(wrap_pyfunction!(get_resolution, m)?)?;
        m.add_function(wrap_pyfunction!(get_sync_rate, m)?)?;
        m.add_function(wrap_pyfunction!(get_count_rate, m)?)?;
        m.add_function(wrap_pyfunction!(get_all_count_rates, m)?)?;
        m.add_function(wrap_pyfunction!(get_flags, m)?)?;
        m.add_function(wrap_pyfunction!(get_elapsed_measurement_time, m)?)?;
        m.add_function(wrap_pyfunction!(get_start_time, m)?)?;
        m.add_function(wrap_pyfunction!(get_warnings, m)?)?;
        m.add_function(wrap_pyfunction!(read_fifo, m)?)?;
        m.add_function(wrap_pyfunction!(is_measurement_running, m)?)?;
        m.add_class::<Mode>()?;
        m.add_class::<RefSource>()?;
        m.add_class::<Edge>()?;
        m.add_class::<MeasurementControl>()?;
    }

    m.add_function(wrap_pyfunction!(parser_py::parse_rs, m)?)?;
    m.add_class::<PtuParser>()?;

    Ok(())
}
