mod mhlib_wrapper;
use mhlib_wrapper::{Edge, Mode, RefSource};
use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;

fn convert_into_py_err(err: String) -> PyErr {
    PyRuntimeError::new_err(err)
}

#[pyfunction]
fn get_library_version() -> PyResult<String> {
    mhlib_wrapper::get_library_version().map_err(convert_into_py_err)
}

#[pyfunction]
fn open_device(device_index: u8) -> PyResult<String> {
    mhlib_wrapper::open_device(device_index).map_err(convert_into_py_err)
}

#[pyfunction]
fn initialize(device_index: u8, mode: Mode, ref_source: RefSource) -> PyResult<()> {
    mhlib_wrapper::initialize(device_index, mode, ref_source).map_err(convert_into_py_err)
}

#[pyfunction]
fn close_device(device_index: u8) -> PyResult<()> {
    mhlib_wrapper::close_device(device_index).map_err(convert_into_py_err)
}

#[pyfunction]
fn get_serial_number(device_index: u8) -> PyResult<String> {
    mhlib_wrapper::get_serial_number(device_index).map_err(convert_into_py_err)
}

#[pyfunction]
fn get_number_of_input_channels(device_index: u8) -> PyResult<i32> {
    mhlib_wrapper::get_number_of_input_channels(device_index).map_err(convert_into_py_err)
}

#[pyfunction]
fn set_sync_divider(device_index: u8, divider: i32) -> PyResult<()> {
    mhlib_wrapper::set_sync_divider(device_index, divider).map_err(convert_into_py_err)
}

#[pyfunction]
fn set_sync_edge_trigger(device_index: u8, trigger_level: i32, mac_edge: Edge) -> PyResult<()> {
    mhlib_wrapper::set_sync_edge_trigger(device_index, trigger_level, mac_edge)
        .map_err(convert_into_py_err)
}

#[pyfunction]
fn set_sync_channel_offset(device_index: u8, offset: i32) -> PyResult<()> {
    mhlib_wrapper::set_sync_channel_offset(device_index, offset).map_err(convert_into_py_err)
}

#[pyfunction]
fn set_input_edge_trigger(
    device_index: u8,
    channel: u8,
    trigger_level: i32,
    mac_edge: Edge,
) -> PyResult<()> {
    mhlib_wrapper::set_input_edge_trigger(device_index, channel, trigger_level, mac_edge)
        .map_err(convert_into_py_err)
}

#[pyfunction]
fn set_input_channel_enable(device_index: u8, channel: u8, enable: bool) -> PyResult<()> {
    mhlib_wrapper::set_input_channel_enable(device_index, channel, enable)
        .map_err(convert_into_py_err)
}

#[pyfunction]
fn set_input_channel_offset(device_index: u8, channel: u8, offset: i32) -> PyResult<()> {
    mhlib_wrapper::set_input_channel_offset(device_index, channel, offset)
        .map_err(convert_into_py_err)
}

#[pyfunction]
fn set_binning(device_index: u8, binning: i32) -> PyResult<()> {
    mhlib_wrapper::set_binning(device_index, binning).map_err(convert_into_py_err)
}

#[pyfunction]
fn set_offset(device_index: u8, offset: i32) -> PyResult<()> {
    mhlib_wrapper::set_offset(device_index, offset).map_err(convert_into_py_err)
}

#[pyfunction]
fn get_resolution(device_index: u8) -> PyResult<f64> {
    mhlib_wrapper::get_resolution(device_index).map_err(convert_into_py_err)
}

#[pyfunction]
fn get_sync_rate(device_index: u8) -> PyResult<i32> {
    mhlib_wrapper::get_sync_rate(device_index).map_err(convert_into_py_err)
}

#[pyfunction]
fn get_count_rate(device_index: u8, channel: u8) -> PyResult<i32> {
    mhlib_wrapper::get_count_rate(device_index, channel).map_err(convert_into_py_err)
}

#[pyfunction]
fn get_warnings(device_index: u8) -> PyResult<String> {
    mhlib_wrapper::get_warnings(device_index).map_err(convert_into_py_err)
}

#[pyfunction]
fn start_measurement(device_index: u8, acquisition_time: i32) -> PyResult<()> {
    mhlib_wrapper::start_measurement(device_index, acquisition_time).map_err(convert_into_py_err)
}

#[pyfunction]
fn stop_measurement(device_index: u8) -> PyResult<()> {
    mhlib_wrapper::stop_measurement(device_index).map_err(convert_into_py_err)
}

#[pyfunction]
fn read_fifo(device_index: u8) -> PyResult<u32> {
    let mut buffer = [0u32; 131072];
    mhlib_wrapper::read_fifo(device_index, buffer.as_mut_ptr()).map_err(convert_into_py_err)
}

#[pyfunction]
fn is_measurement_running(device_index: u8) -> PyResult<bool> {
    mhlib_wrapper::is_measurement_running(device_index).map_err(convert_into_py_err)
}

#[pymodule]
#[pyo3(name = "_mhtk_rs")]
fn _mhtk_rs(_py: Python, m: &PyModule) -> PyResult<()> {
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

    Ok(())
}
