use pyo3::prelude::*;

#[cfg(any(
    all(target_arch = "x86_64", target_os = "windows"),
    all(target_arch = "x86_64", target_os = "linux")
))]
mod mhlib_wrapper;

#[cfg(any(
    all(target_arch = "x86_64", target_os = "windows"),
    all(target_arch = "x86_64", target_os = "linux")
))]
mod mhlib_wrapper_py;

#[cfg(any(
    all(target_arch = "x86_64", target_os = "windows"),
    all(target_arch = "x86_64", target_os = "linux")
))]
use mhlib_wrapper_py::*;

mod parser;
mod parser_py;

#[pymodule]
#[pyo3(name = "_mhtk_rs")]
fn _mhtk_rs(_py: Python, m: &PyModule) -> PyResult<()> {
    #[cfg(any(
        all(target_arch = "x86_64", target_os = "windows"),
        all(target_arch = "x86_64", target_os = "linux")
    ))]
    {
        m.add_function(wrap_pyfunction!(get_library_version, m)?)?;
        m.add_function(wrap_pyfunction!(open_device, m)?)?;
        m.add_function(wrap_pyfunction!(initialize, m)?)?;
        m.add_function(wrap_pyfunction!(close_device, m)?)?;
        m.add_function(wrap_pyfunction!(get_serial_number, m)?)?;
        m.add_function(wrap_pyfunction!(get_number_of_input_channels, m)?)?;
        m.add_function(wrap_pyfunction!(set_sync_divider, m)?)?;
        m.add_function(wrap_pyfunction!(set_sync_edge_trigger, m)?)?;
        m.add_function(wrap_pyfunction!(set_sync_channel_offset, m)?)?;
        m.add_function(wrap_pyfunction!(set_input_edge_trigger, m)?)?;
        m.add_function(wrap_pyfunction!(set_input_channel_enable, m)?)?;
        m.add_function(wrap_pyfunction!(set_input_channel_offset, m)?)?;
        m.add_function(wrap_pyfunction!(set_binning, m)?)?;
        m.add_function(wrap_pyfunction!(set_offset, m)?)?;
        m.add_function(wrap_pyfunction!(get_resolution, m)?)?;
        m.add_function(wrap_pyfunction!(get_sync_rate, m)?)?;
        m.add_function(wrap_pyfunction!(get_count_rate, m)?)?;
        m.add_function(wrap_pyfunction!(get_warnings, m)?)?;
        m.add_function(wrap_pyfunction!(start_measurement, m)?)?;
        m.add_function(wrap_pyfunction!(stop_measurement, m)?)?;
        m.add_function(wrap_pyfunction!(read_fifo, m)?)?;
        m.add_function(wrap_pyfunction!(is_measurement_running, m)?)?;
    }

    m.add_function(wrap_pyfunction!(parser_py::parse_rs, m)?)?;

    Ok(())
}
