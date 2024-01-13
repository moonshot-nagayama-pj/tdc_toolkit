use crate::mhlib_wrapper::{self, MeasurementControl};
use crate::mhlib_wrapper::{Edge, Mode, RefSource};
use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;

fn convert_into_py_err(err: String) -> PyErr {
    PyRuntimeError::new_err(err)
}

#[pyfunction]
pub fn get_library_version() -> PyResult<String> {
    mhlib_wrapper::get_library_version().map_err(convert_into_py_err)
}

#[pyfunction]
pub fn open_device(device_index: u8) -> PyResult<String> {
    mhlib_wrapper::open_device(device_index).map_err(convert_into_py_err)
}

#[pyfunction]
pub fn close_device(device_index: u8) -> PyResult<()> {
    mhlib_wrapper::close_device(device_index).map_err(convert_into_py_err)
}

#[pyfunction]
pub fn initialize(device_index: u8, mode: Mode, ref_source: RefSource) -> PyResult<()> {
    mhlib_wrapper::initialize(device_index, mode, ref_source).map_err(convert_into_py_err)
}

#[pyfunction]
pub fn get_hardware_info(device_index: u8) -> PyResult<(String, String, String)> {
    mhlib_wrapper::get_hardware_info(device_index).map_err(convert_into_py_err)
}

#[pyfunction]
pub fn get_feature(device_index: u8) -> PyResult<i32> {
    mhlib_wrapper::get_feature(device_index).map_err(convert_into_py_err)
}

#[pyfunction]
pub fn get_serial_number(device_index: u8) -> PyResult<String> {
    mhlib_wrapper::get_serial_number(device_index).map_err(convert_into_py_err)
}

#[pyfunction]
pub fn get_base_resolution(device_index: u8) -> PyResult<(f64, i32)> {
    mhlib_wrapper::get_base_resolution(device_index).map_err(convert_into_py_err)
}

#[pyfunction]
pub fn get_number_of_input_channels(device_index: u8) -> PyResult<i32> {
    mhlib_wrapper::get_number_of_input_channels(device_index).map_err(convert_into_py_err)
}

#[pyfunction]
pub fn get_number_of_modules(device_index: u8) -> PyResult<i32> {
    mhlib_wrapper::get_number_of_modules(device_index).map_err(convert_into_py_err)
}

#[pyfunction]
pub fn get_module_info(device_index: u8, module_index: u8) -> PyResult<(i32, i32)> {
    mhlib_wrapper::get_module_info(device_index, module_index).map_err(convert_into_py_err)
}

#[pyfunction]
pub fn get_debug_info(device_index: u8) -> PyResult<String> {
    mhlib_wrapper::get_debug_info(device_index).map_err(convert_into_py_err)
}

#[pyfunction]
pub fn set_sync_divider(device_index: u8, divider: i32) -> PyResult<()> {
    mhlib_wrapper::set_sync_divider(device_index, divider).map_err(convert_into_py_err)
}

#[pyfunction]
pub fn set_sync_edge_trigger(device_index: u8, trigger_level: i32, mac_edge: Edge) -> PyResult<()> {
    mhlib_wrapper::set_sync_edge_trigger(device_index, trigger_level, mac_edge)
        .map_err(convert_into_py_err)
}

#[pyfunction]
pub fn set_sync_channel_offset(device_index: u8, offset: i32) -> PyResult<()> {
    mhlib_wrapper::set_sync_channel_offset(device_index, offset).map_err(convert_into_py_err)
}

#[pyfunction]
pub fn set_sync_channel_enable(device_index: u8, enable: bool) -> PyResult<()> {
    mhlib_wrapper::set_sync_channel_enable(device_index, enable).map_err(convert_into_py_err)
}

#[pyfunction]
pub fn set_sync_deadtime(device_index: u8, on: bool, deadtime_ps: i32) -> PyResult<()> {
    mhlib_wrapper::set_sync_deadtime(device_index, on, deadtime_ps).map_err(convert_into_py_err)
}

#[pyfunction]
pub fn set_input_edge_trigger(
    device_index: u8,
    channel: u8,
    trigger_level: i32,
    mac_edge: Edge,
) -> PyResult<()> {
    mhlib_wrapper::set_input_edge_trigger(device_index, channel, trigger_level, mac_edge)
        .map_err(convert_into_py_err)
}

#[pyfunction]
pub fn set_input_channel_offset(device_index: u8, channel: u8, offset: i32) -> PyResult<()> {
    mhlib_wrapper::set_input_channel_offset(device_index, channel, offset)
        .map_err(convert_into_py_err)
}

#[pyfunction]
pub fn set_input_channel_enable(device_index: u8, channel: u8, enable: bool) -> PyResult<()> {
    mhlib_wrapper::set_input_channel_enable(device_index, channel, enable)
        .map_err(convert_into_py_err)
}

#[pyfunction]
pub fn set_input_deadtime(
    device_index: u8,
    channel: u8,
    on: bool,
    deadtime_ps: i32,
) -> PyResult<()> {
    mhlib_wrapper::set_input_deadtime(device_index, channel, on, deadtime_ps)
        .map_err(convert_into_py_err)
}

#[pyfunction]
pub fn set_input_hysteresis(device_index: u8, hist_code: u8) -> PyResult<()> {
    mhlib_wrapper::set_input_hysteresis(device_index, hist_code).map_err(convert_into_py_err)
}

#[pyfunction]
pub fn set_stop_overflow(device_index: u8, stop_overflow: bool, stop_count: u32) -> PyResult<()> {
    mhlib_wrapper::set_stop_overflow(device_index, stop_overflow, stop_count)
        .map_err(convert_into_py_err)
}

#[pyfunction]
pub fn set_binning(device_index: u8, binning: i32) -> PyResult<()> {
    mhlib_wrapper::set_binning(device_index, binning).map_err(convert_into_py_err)
}

#[pyfunction]
pub fn set_offset(device_index: u8, offset: i32) -> PyResult<()> {
    mhlib_wrapper::set_offset(device_index, offset).map_err(convert_into_py_err)
}

#[pyfunction]
pub fn set_histogram_length(device_index: u8, len_code: i32) -> PyResult<i32> {
    mhlib_wrapper::set_histogram_length(device_index, len_code).map_err(convert_into_py_err)
}

#[pyfunction]
pub fn clear_histogram_memory(device_index: u8) -> PyResult<()> {
    mhlib_wrapper::clear_histogram_memory(device_index).map_err(convert_into_py_err)
}

#[pyfunction]
pub fn set_measurement_control(
    device_index: u8,
    meas_control: MeasurementControl,
    start_edge: Edge,
    stop_edge: Edge,
) -> PyResult<()> {
    mhlib_wrapper::set_measurement_control(device_index, meas_control, start_edge, stop_edge)
        .map_err(convert_into_py_err)
}

#[pyfunction]
pub fn set_trigger_output(device_index: u8, period_100ns: i32) -> PyResult<()> {
    mhlib_wrapper::set_trigger_output(device_index, period_100ns).map_err(convert_into_py_err)
}

#[pyfunction]
pub fn start_measurement(device_index: u8, acquisition_time: i32) -> PyResult<()> {
    mhlib_wrapper::start_measurement(device_index, acquisition_time).map_err(convert_into_py_err)
}

#[pyfunction]
pub fn stop_measurement(device_index: u8) -> PyResult<()> {
    mhlib_wrapper::stop_measurement(device_index).map_err(convert_into_py_err)
}

#[pyfunction]
pub fn ctc_status(device_index: u8) -> PyResult<i32> {
    mhlib_wrapper::ctc_status(device_index).map_err(convert_into_py_err)
}
#[pyfunction]
pub fn get_histogram(device_index: u8, channel: u8) -> PyResult<Vec<u32>> {
    mhlib_wrapper::get_histogram(device_index, channel).map_err(convert_into_py_err)
}

#[pyfunction]
pub fn get_all_histograms(device_index: u8) -> PyResult<Vec<u32>> {
    mhlib_wrapper::get_all_histogram(device_index).map_err(convert_into_py_err)
}

#[pyfunction]
pub fn get_resolution(device_index: u8) -> PyResult<f64> {
    mhlib_wrapper::get_resolution(device_index).map_err(convert_into_py_err)
}

#[pyfunction]
pub fn get_sync_rate(device_index: u8) -> PyResult<i32> {
    mhlib_wrapper::get_sync_rate(device_index).map_err(convert_into_py_err)
}

#[pyfunction]
pub fn get_count_rate(device_index: u8, channel: u8) -> PyResult<i32> {
    mhlib_wrapper::get_count_rate(device_index, channel).map_err(convert_into_py_err)
}

#[pyfunction]
pub fn get_all_count_rates(device_index: u8) -> PyResult<(i32, Vec<i32>)> {
    mhlib_wrapper::get_all_count_rates(device_index).map_err(convert_into_py_err)
}

#[pyfunction]
pub fn get_flags(device_index: u8) -> PyResult<i32> {
    mhlib_wrapper::get_flags(device_index).map_err(convert_into_py_err)
}

#[pyfunction]
pub fn get_elapsed_measurement_time(device_index: u8) -> PyResult<f64> {
    mhlib_wrapper::get_elapsed_measurement_time(device_index).map_err(convert_into_py_err)
}

#[pyfunction]
pub fn get_start_time(device_index: u8) -> PyResult<(u32, u32, u32)> {
    mhlib_wrapper::get_start_time(device_index).map_err(convert_into_py_err)
}

#[pyfunction]
pub fn get_warnings(device_index: u8) -> PyResult<String> {
    mhlib_wrapper::get_warnings(device_index).map_err(convert_into_py_err)
}

#[pyfunction]
pub fn read_fifo(device_index: u8) -> PyResult<u32> {
    let mut buffer = [0u32; 131072];
    mhlib_wrapper::read_fifo(device_index, buffer.as_mut_ptr()).map_err(convert_into_py_err)
}

#[pyfunction]
pub fn is_measurement_running(device_index: u8) -> PyResult<bool> {
    mhlib_wrapper::is_measurement_running(device_index).map_err(convert_into_py_err)
}
