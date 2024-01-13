use pyo3::prelude::*;
use std::os::raw::c_int;

mod bindings {
    #![allow(dead_code)]
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

use self::bindings::*;

#[derive(Clone)]
#[repr(i32)]
#[pyclass]
pub enum Mode {
    T2 = MODE_T2 as i32,
    T3 = MODE_T3 as i32,
}

#[derive(Clone)]
#[repr(u32)]
#[pyclass]
pub enum RefSource {
    InternalClock = REFSRC_INTERNAL,
    ExternalClock10MHz = REFSRC_EXTERNAL_10MHZ,
    WhiteRabbitMaster = REFSRC_WR_MASTER_GENERIC,
    WhiteRabbitSlave = REFSRC_WR_SLAVE_GENERIC,
    WhiteRabbitGrandMaster = REFSRC_WR_GRANDM_GENERIC,
    ExternalGpsPps = REFSRC_EXTN_GPS_PPS,
    ExternalGpsPPsUart = REFSRC_EXTN_GPS_PPS_UART,
    WhiteRabbitMasterMultiHarp = REFSRC_WR_MASTER_MHARP,
    WhiteRabbitSlaveMultiHarp = REFSRC_WR_SLAVE_MHARP,
    WhiteRabbitGrandMasterMultiHarp = REFSRC_WR_GRANDM_MHARP,
}

#[derive(Clone)]
#[repr(i32)]
#[pyclass]
pub enum Edge {
    Falling = EDGE_FALLING as i32,
    Rising = EDGE_RISING as i32,
}

#[derive(Clone)]
#[repr(i32)]
#[pyclass]
pub enum MeasurementControl {
    SingleShotCtc = MEASCTRL_SINGLESHOT_CTC as i32,
    C1Gated = MEASCTRL_C1_GATED as i32,
    C1StartCtcStop = MEASCTRL_C1_START_CTC_STOP as i32,
    C1StartC2Stop = MEASCTRL_C1_START_C2_STOP as i32,
    WhiteRabbitM2S = MEASCTRL_WR_M2S as i32,
    WhiteRabbitS2M = MEASCTRL_WR_S2M as i32,
    SwitchStartSwitchStop = MEASCTRL_SW_START_SW_STOP as i32,
}

fn handle_error(ret: c_int) -> Result<(), String> {
    let mut error_string: [u8; 40] = [0; 40];
    if ret != 0 {
        unsafe {
            MH_GetErrorString(error_string.as_mut_ptr() as *mut i8, ret);
        }
        return Err(convert_into_string(&error_string));
    }
    Ok(())
}

fn convert_into_string(vec: &[u8]) -> String {
    let s = std::str::from_utf8(vec).unwrap();
    s.trim_matches('\0').to_string()
}

pub fn get_library_version() -> Result<String, String> {
    unsafe {
        let mut ver_str: [u8; 8] = [0; 8];
        let ret = MH_GetLibraryVersion(ver_str.as_mut_ptr() as *mut i8);
        handle_error(ret)?;
        Ok(convert_into_string(&ver_str))
    }
}

pub fn open_device(device_index: u8) -> Result<String, String> {
    let mut vec_serial: [u8; 9] = [0; 9];
    unsafe {
        let ret = MH_OpenDevice(device_index.into(), vec_serial.as_mut_ptr() as *mut i8);
        println!("ret: {}", ret);
        handle_error(ret)?;
        Ok(convert_into_string(&vec_serial))
    }
}

pub fn close_device(device_index: u8) -> Result<(), String> {
    unsafe {
        let ret = MH_CloseDevice(device_index.into());
        handle_error(ret)?;
        Ok(())
    }
}

pub fn initialize(device_index: u8, mode: Mode, ref_source: RefSource) -> Result<(), String> {
    unsafe {
        let ret = MH_Initialize(device_index.into(), mode as i32, ref_source as i32);
        handle_error(ret)?;
        Ok(())
    }
}

pub fn get_hardware_info(device_index: u8) -> Result<(String, String, String), String> {
    let mut model_vec: [u8; 24] = [0; 24];
    let mut partno_vec: [u8; 8] = [0; 8];
    let mut version_vec: [u8; 8] = [0; 8];
    unsafe {
        let ret = MH_GetHardwareInfo(
            device_index.into(),
            model_vec.as_mut_ptr() as *mut i8,
            partno_vec.as_mut_ptr() as *mut i8,
            version_vec.as_mut_ptr() as *mut i8,
        );
        handle_error(ret)?;
        Ok((
            convert_into_string(&model_vec),
            convert_into_string(&partno_vec),
            convert_into_string(&version_vec),
        ))
    }
}

pub fn get_feature(device_index: u8) -> Result<i32, String> {
    let mut features = 0i32;
    unsafe {
        let ret = MH_GetFeatures(device_index.into(), &mut features);
        handle_error(ret)?;
        Ok(features)
    }
}

pub fn get_serial_number(device_index: u8) -> Result<String, String> {
    let mut vec_serial: [u8; 9] = [0; 9];
    unsafe {
        let ret = MH_GetSerialNumber(device_index.into(), vec_serial.as_mut_ptr() as *mut i8);
        handle_error(ret)?;
        Ok(convert_into_string(&vec_serial))
    }
}
pub fn get_base_resolution(device_index: u8) -> Result<(f64, i32), String> {
    let mut resolution: f64 = 0.0;
    let mut bin_steps: i32 = 0;
    unsafe {
        let ret = MH_GetBaseResolution(device_index.into(), &mut resolution, &mut bin_steps);
        handle_error(ret)?;
        Ok((resolution, bin_steps))
    }
}

pub fn get_number_of_input_channels(device_index: u8) -> Result<i32, String> {
    let mut num_channels: i32 = 0;
    unsafe {
        let ret = MH_GetNumOfInputChannels(device_index.into(), &mut num_channels);
        handle_error(ret)?;
        Ok(num_channels)
    }
}

pub fn get_number_of_modules(device_index: u8) -> Result<i32, String> {
    let mut number_of_modules = 0;
    unsafe {
        let ret = MH_GetNumOfModules(device_index.into(), &mut number_of_modules);
        handle_error(ret)?;
        Ok(number_of_modules)
    }
}

pub fn get_module_info(device_index: u8, module_index: u8) -> Result<(i32, i32), String> {
    let mut model_code = 0i32;
    let mut version_code = 0i32;
    unsafe {
        let ret = MH_GetModuleInfo(
            device_index.into(),
            module_index.into(),
            &mut model_code,
            &mut version_code,
        );
        handle_error(ret)?;
        Ok((model_code, version_code))
    }
}

pub fn get_debug_info(device_index: u8) -> Result<String, String> {
    let mut debug_info: [u8; 65536] = [0; 65536];
    unsafe {
        let ret = MH_GetDebugInfo(device_index.into(), debug_info.as_mut_ptr() as *mut i8);
        handle_error(ret)?;
        Ok(convert_into_string(&debug_info))
    }
}

pub fn set_sync_divider(device_index: u8, divider: i32) -> Result<(), String> {
    unsafe {
        let ret = MH_SetSyncDiv(device_index.into(), divider);
        handle_error(ret)?;
        Ok(())
    }
}
pub fn set_sync_edge_trigger(
    device_index: u8,
    trigger_level: i32,
    mac_edge: Edge,
) -> Result<(), String> {
    unsafe {
        let ret = MH_SetSyncEdgeTrg(device_index.into(), trigger_level, mac_edge as i32);
        handle_error(ret)?;
        Ok(())
    }
}

pub fn set_sync_channel_offset(device_index: u8, sync_timing_offset: i32) -> Result<(), String> {
    unsafe {
        let ret = MH_SetSyncChannelOffset(device_index.into(), sync_timing_offset);
        handle_error(ret)?;
        Ok(())
    }
}

pub fn set_sync_channel_enable(device_index: u8, enable: bool) -> Result<(), String> {
    unsafe {
        let ret = MH_SetSyncChannelEnable(device_index.into(), if enable { 1 } else { 0 });
        handle_error(ret)?;
        Ok(())
    }
}

pub fn set_sync_deadtime(device_index: u8, on: bool, deadtime_ps: i32) -> Result<(), String> {
    unsafe {
        let ret = MH_SetSyncDeadTime(device_index.into(), if on { 1 } else { 0 }, deadtime_ps);
        handle_error(ret)?;
        Ok(())
    }
}

pub fn set_input_edge_trigger(
    device_index: u8,
    channel: u8,
    level: i32,
    mac_edge: Edge,
) -> Result<(), String> {
    unsafe {
        let ret = MH_SetInputEdgeTrg(device_index.into(), channel.into(), level, mac_edge as i32);
        handle_error(ret)?;
        Ok(())
    }
}

pub fn set_input_channel_offset(device_index: u8, channel: u8, offset: i32) -> Result<(), String> {
    unsafe {
        let ret = MH_SetInputChannelOffset(device_index.into(), channel.into(), offset);
        handle_error(ret)?;
        Ok(())
    }
}
pub fn set_input_channel_enable(device_index: u8, channel: u8, enable: bool) -> Result<(), String> {
    unsafe {
        let ret = MH_SetInputChannelEnable(device_index.into(), channel.into(), enable as i32);
        handle_error(ret)?;
        Ok(())
    }
}

pub fn set_input_deadtime(
    device_index: u8,
    channel: u8,
    on: bool,
    deadtime_ps: i32,
) -> Result<(), String> {
    unsafe {
        let ret = MH_SetInputDeadTime(
            device_index.into(),
            channel.into(),
            if on { 1 } else { 0 },
            deadtime_ps,
        );
        handle_error(ret)?;
        Ok(())
    }
}

pub fn set_input_hysteresis(device_index: u8, hyst_code: u8) -> Result<(), String> {
    unsafe {
        let ret = MH_SetInputHysteresis(device_index.into(), hyst_code.into());
        handle_error(ret)?;
        Ok(())
    }
}

pub fn set_stop_overflow(
    device_index: u8,
    stop_overflow: bool,
    stop_count: u32,
) -> Result<(), String> {
    unsafe {
        let ret = MH_SetStopOverflow(
            device_index.into(),
            if stop_overflow { 1 } else { 0 },
            stop_count,
        );
        handle_error(ret)?;
        Ok(())
    }
}
pub fn set_binning(device_index: u8, binning: i32) -> Result<(), String> {
    unsafe {
        let ret = MH_SetBinning(device_index.into(), binning);
        handle_error(ret)?;
        Ok(())
    }
}

pub fn set_offset(device_index: u8, offset: i32) -> Result<(), String> {
    unsafe {
        let ret = MH_SetOffset(device_index.into(), offset);
        handle_error(ret)?;
        Ok(())
    }
}

pub fn set_histogram_length(device_index: u8, len_code: i32) -> Result<i32, String> {
    let mut actual_length = 0i32;
    unsafe {
        let ret = MH_SetHistoLen(device_index.into(), len_code, &mut actual_length);
        handle_error(ret)?;
        Ok(actual_length)
    }
}

pub fn clear_histogram_memory(device_index: u8) -> Result<(), String> {
    unsafe {
        let ret = MH_ClearHistMem(device_index.into());
        handle_error(ret)?;
        Ok(())
    }
}

pub fn set_measurement_control(
    device_index: u8,
    meas_control: MeasurementControl,
    start_edge: Edge,
    stop_edge: Edge,
) -> Result<(), String> {
    unsafe {
        let ret = MH_SetMeasControl(
            device_index.into(),
            meas_control as i32,
            start_edge as i32,
            stop_edge as i32,
        );
        handle_error(ret)?;
        Ok(())
    }
}

pub fn set_trigger_output(device_index: u8, period_100ns: i32) -> Result<(), String> {
    unsafe {
        let ret = MH_SetTriggerOutput(device_index.into(), period_100ns);
        handle_error(ret)?;
        Ok(())
    }
}

pub fn start_measurement(device_index: u8, acquisition_time: i32) -> Result<(), String> {
    unsafe {
        let ret = MH_StartMeas(device_index.into(), acquisition_time);
        handle_error(ret)?;
        Ok(())
    }
}

pub fn stop_measurement(device_index: u8) -> Result<(), String> {
    unsafe {
        let ret = MH_StopMeas(device_index.into());
        handle_error(ret)?;
        Ok(())
    }
}

pub fn ctc_status(device_index: u8) -> Result<i32, String> {
    let mut ctc_status_val = 0i32;
    unsafe {
        let ret = MH_CTCStatus(device_index.into(), &mut ctc_status_val);
        handle_error(ret)?;
        Ok(ctc_status_val)
    }
}

pub fn get_histogram(device_index: u8, channel: u8) -> Result<Vec<u32>, String> {
    let mut histogram_vec = [0u32; 65536];
    unsafe {
        let ret = MH_GetHistogram(
            device_index.into(),
            histogram_vec.as_mut_ptr(),
            channel.into(),
        );
        handle_error(ret)?;
        Ok(histogram_vec.to_vec())
    }
}
pub fn get_all_histogram(device_index: u8) -> Result<Vec<u32>, String> {
    let mut histogram_vec = [0u32; 65536];
    unsafe {
        let ret = MH_GetAllHistograms(device_index.into(), histogram_vec.as_mut_ptr());
        handle_error(ret)?;
        Ok(histogram_vec.to_vec())
    }
}

pub fn get_resolution(device_index: u8) -> Result<f64, String> {
    let mut resolution: f64 = 0.0;
    unsafe {
        let ret = MH_GetResolution(device_index.into(), &mut resolution);
        handle_error(ret)?;
        Ok(resolution)
    }
}

pub fn get_sync_rate(device_index: u8) -> Result<i32, String> {
    let mut sync_rate: i32 = 0;
    unsafe {
        let ret = MH_GetSyncRate(device_index.into(), &mut sync_rate);
        handle_error(ret)?;
        Ok(sync_rate)
    }
}
pub fn get_count_rate(device_index: u8, channel: u8) -> Result<i32, String> {
    let mut count_rate: i32 = 0;
    unsafe {
        let ret = MH_GetCountRate(device_index.into(), channel.into(), &mut count_rate);
        handle_error(ret)?;
        Ok(count_rate)
    }
}

pub fn get_all_count_rates(device_index: u8) -> Result<(i32, Vec<i32>), String> {
    let mut sync_rate: i32 = 0;
    let mut count_rates = [0i32; MAXINPCHAN as usize];
    unsafe {
        let ret = MH_GetAllCountRates(
            device_index.into(),
            &mut sync_rate,
            count_rates.as_mut_ptr(),
        );
        handle_error(ret)?;
        Ok((sync_rate, count_rates.to_vec()))
    }
}

pub fn get_flags(device_index: u8) -> Result<i32, String> {
    let mut flags = 0i32;
    unsafe {
        let ret = MH_GetFlags(device_index.into(), &mut flags);
        handle_error(ret)?;
        Ok(flags)
    }
}

pub fn get_elapsed_measurement_time(device_index: u8) -> Result<f64, String> {
    let mut elapsed_time = 0f64;
    unsafe {
        let ret = MH_GetElapsedMeasTime(device_index.into(), &mut elapsed_time);
        handle_error(ret)?;
        Ok(elapsed_time)
    }
}

pub fn get_start_time(device_index: u8) -> Result<(u32, u32, u32), String> {
    let mut time2 = 0u32;
    let mut time1 = 0u32;
    let mut time0 = 0u32;
    unsafe {
        let ret = MH_GetStartTime(device_index.into(), &mut time2, &mut time1, &mut time0);
        handle_error(ret)?;
        Ok((time2, time1, time0))
    }
}

pub fn get_warnings(device_index: u8) -> Result<String, String> {
    let mut warnings: i32 = 0;
    let mut text = [0u8; 16384];
    unsafe {
        let ret = MH_GetWarnings(device_index.into(), &mut warnings);
        handle_error(ret)?;
        let ret = MH_GetWarningsText(device_index.into(), text.as_mut_ptr() as *mut i8, warnings);
        handle_error(ret)?;
        Ok(convert_into_string(&text))
    }
}

pub fn read_fifo(device_index: u8, buffer: *mut u32) -> Result<u32, String> {
    let mut num_records: i32 = 0;
    unsafe {
        let ret = MH_ReadFiFo(device_index.into(), buffer, &mut num_records);
        handle_error(ret)?;
        Ok(num_records as u32)
    }
}
pub fn is_measurement_running(device_index: u8) -> Result<bool, String> {
    let mut ctc_status: i32 = 0;
    unsafe {
        let ret = MH_CTCStatus(device_index.into(), &mut ctc_status);
        handle_error(ret)?;
        Ok(ctc_status == 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_lib_version() {
        assert_eq!(get_library_version().unwrap(), String::from("3.1"));
    }
    #[test]
    fn test_open_device() {
        assert_eq!(
            open_device(0),
            Err(String::from("MH_ERROR_DEVICE_OPEN_FAIL"))
        );
    }
}
