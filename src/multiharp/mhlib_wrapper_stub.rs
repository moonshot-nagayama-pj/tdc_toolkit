use anyhow::Result;

use super::mhlib_wrapper_header::{Edge, MeasurementControl, Mode, RefSource};

pub fn get_library_version() -> Result<String> {
    Ok("Stub".to_string())
}

pub fn open_device(_device_index: u8) -> Result<String> {
    Ok("ABC123".to_string())
}

pub fn close_device(_device_index: u8) -> Result<()> {
    Ok(())
}

pub fn initialize(_device_index: u8, _mode: Mode, _ref_source: RefSource) -> Result<()> {
    Ok(())
}

pub fn get_hardware_info(_device_index: u8) -> Result<(String, String, String)> {
    Ok((
        "model".to_string(),
        "part_no".to_string(),
        "version".to_string(),
    ))
}

pub fn get_feature(_device_index: u8) -> Result<i32> {
    Ok(0i32)
}

pub fn get_serial_number(_device_index: u8) -> Result<String> {
    Ok("ABC123".to_string())
}

pub fn get_base_resolution(_device_index: u8) -> Result<(f64, i32)> {
    Ok((5.0_f64, 0_i32))
}

pub fn get_number_of_input_channels(_device_index: u8) -> Result<i32> {
    Ok(8_i32)
}

pub fn get_number_of_modules(_device_index: u8) -> Result<i32> {
    Ok(3_i32)
}

pub fn get_module_info(_device_index: u8, _module_index: u8) -> Result<(i32, i32)> {
    Ok((0i32, 0i32))
}

pub fn get_debug_info(_device_index: u8) -> Result<String> {
    Ok("debug_info".to_string())
}

pub fn set_sync_divider(_device_index: u8, _divider: i32) -> Result<()> {
    Ok(())
}

pub fn set_sync_edge_trigger(
    _device_index: u8,
    _trigger_level: i32,
    _mac_edge: Edge,
) -> Result<()> {
    Ok(())
}

pub fn set_sync_channel_offset(_device_index: u8, _sync_timing_offset: i32) -> Result<()> {
    Ok(())
}

pub fn set_sync_channel_enable(_device_index: u8, _enable: bool) -> Result<()> {
    Ok(())
}

pub fn set_sync_deadtime(_device_index: u8, _on: bool, _deadtime_ps: i32) -> Result<()> {
    Ok(())
}

pub fn set_input_edge_trigger(
    _device_index: u8,
    _channel: u8,
    _level: i32,
    _mac_edge: Edge,
) -> Result<()> {
    Ok(())
}

pub fn set_input_channel_offset(_device_index: u8, _channel: u8, _offset: i32) -> Result<()> {
    Ok(())
}

pub fn set_input_channel_enable(_device_index: u8, _channel: u8, _enable: bool) -> Result<()> {
    Ok(())
}

pub fn set_input_deadtime(
    _device_index: u8,
    _channel: u8,
    _on: bool,
    _deadtime_ps: i32,
) -> Result<()> {
    Ok(())
}

pub fn set_input_hysteresis(_device_index: u8, _hyst_code: u8) -> Result<()> {
    Ok(())
}

pub fn set_stop_overflow(_device_index: u8, _stop_overflow: bool, _stop_count: u32) -> Result<()> {
    Ok(())
}

pub fn set_binning(_device_index: u8, _binning: i32) -> Result<()> {
    Ok(())
}

pub fn set_offset(_device_index: u8, _offset: i32) -> Result<()> {
    Ok(())
}

pub fn set_histogram_length(_device_index: u8, _len_code: i32) -> Result<i32> {
    Ok(5i32)
}

pub fn clear_histogram_memory(_device_index: u8) -> Result<()> {
    Ok(())
}

pub fn set_measurement_control(
    _device_index: u8,
    _meas_control: MeasurementControl,
    _start_edge: Edge,
    _stop_edge: Edge,
) -> Result<()> {
    Ok(())
}

pub fn set_trigger_output(_device_index: u8, _period_100ns: i32) -> Result<()> {
    Ok(())
}

pub fn start_measurement(_device_index: u8, _acquisition_time: i32) -> Result<()> {
    Ok(())
}

pub fn stop_measurement(_device_index: u8) -> Result<()> {
    Ok(())
}

pub fn ctc_status(_device_index: u8) -> Result<i32> {
    Ok(0i32)
}

pub fn get_histogram(_device_index: u8, _channel: u8) -> Result<Vec<u32>> {
    let histogram_vec = [0u32; 65536];
    Ok(histogram_vec.to_vec())
}

pub fn get_all_histogram(_device_index: u8) -> Result<Vec<u32>> {
    let histogram_vec = [0u32; 65536];
    Ok(histogram_vec.to_vec())
}

pub fn get_resolution(_device_index: u8) -> Result<f64> {
    Ok(0.0f64)
}

pub fn get_sync_rate(_device_index: u8) -> Result<i32> {
    Ok(0i32)
}
pub fn get_count_rate(_device_index: u8, _channel: u8) -> Result<i32> {
    Ok(0i32)
}

pub fn get_all_count_rates(_device_index: u8) -> Result<(i32, Vec<i32>)> {
    let count_rates = [0i32; 64 as usize];
    Ok((0i32, count_rates.to_vec()))
}

pub fn get_flags(_device_index: u8) -> Result<i32> {
    Ok(0i32)
}

pub fn get_elapsed_measurement_time(_device_index: u8) -> Result<f64> {
    Ok(0f64)
}

pub fn get_start_time(_device_index: u8) -> Result<(u32, u32, u32)> {
    Ok((0u32, 0u32, 0u32))
}

pub fn get_warnings(_device_index: u8) -> Result<String> {
    Ok("warning".to_string())
}

pub fn read_fifo(_device_index: u8) -> Result<Vec<u32>> {
    Ok(vec![0u32])
}

pub fn is_measurement_running(_device_index: u8) -> Result<bool> {
    Ok(true)
}
