use anyhow::{Result, anyhow};
use std::os::raw::c_int;

mod bindings {
    #![allow(dead_code, clippy::unreadable_literal)]
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

use self::bindings::{
    MAXINPCHAN, MH_CTCStatus, MH_ClearHistMem, MH_CloseDevice, MH_GetAllCountRates,
    MH_GetAllHistograms, MH_GetBaseResolution, MH_GetCountRate, MH_GetDebugInfo,
    MH_GetElapsedMeasTime, MH_GetErrorString, MH_GetFeatures, MH_GetFlags, MH_GetHardwareInfo,
    MH_GetHistogram, MH_GetLibraryVersion, MH_GetModuleInfo, MH_GetNumOfInputChannels,
    MH_GetNumOfModules, MH_GetResolution, MH_GetSerialNumber, MH_GetStartTime, MH_GetSyncRate,
    MH_GetWarnings, MH_GetWarningsText, MH_Initialize, MH_OpenDevice, MH_ReadFiFo, MH_SetBinning,
    MH_SetHistoLen, MH_SetInputChannelEnable, MH_SetInputChannelOffset, MH_SetInputDeadTime,
    MH_SetInputEdgeTrg, MH_SetInputHysteresis, MH_SetMeasControl, MH_SetOffset, MH_SetStopOverflow,
    MH_SetSyncChannelEnable, MH_SetSyncChannelOffset, MH_SetSyncDeadTime, MH_SetSyncDiv,
    MH_SetSyncEdgeTrg, MH_SetTriggerOutput, MH_StartMeas, MH_StopMeas,
};

use super::mhlib_wrapper_header;
use super::mhlib_wrapper_header::{
    Edge, MH160InternalChannelId, MeasurementControl, Mode, RefSource,
};

fn handle_error(ret: c_int) -> Result<()> {
    let mut error_string: [u8; 40] = [0; 40];
    if ret != 0 {
        unsafe {
            MH_GetErrorString(error_string.as_mut_ptr().cast::<i8>(), ret);
        }
        return Err(anyhow!(convert_into_string(&error_string)));
    }
    Ok(())
}

fn convert_into_string(vec: &[u8]) -> String {
    let s = match std::str::from_utf8(vec) {
        Ok(output) => output.to_string(),
        Err(e) => e.to_string(),
    };
    s.trim_matches('\0').to_string()
}

pub fn get_library_version() -> Result<String> {
    unsafe {
        let mut ver_str: [u8; 8] = [0; 8];
        let ret = MH_GetLibraryVersion(ver_str.as_mut_ptr().cast::<i8>());
        handle_error(ret)?;
        Ok(convert_into_string(&ver_str))
    }
}

pub fn open_device(device_index: u8) -> Result<String> {
    let mut vec_serial: [u8; 9] = [0; 9];
    unsafe {
        let ret = MH_OpenDevice(device_index.into(), vec_serial.as_mut_ptr().cast::<i8>());
        handle_error(ret)?;
        Ok(convert_into_string(&vec_serial))
    }
}

pub fn close_device(device_index: u8) -> Result<()> {
    unsafe {
        let ret = MH_CloseDevice(device_index.into());
        handle_error(ret)?;
        Ok(())
    }
}

pub fn initialize(device_index: u8, mode: Mode, ref_source: RefSource) -> Result<()> {
    unsafe {
        let ret = MH_Initialize(device_index.into(), mode as i32, ref_source as i32);
        handle_error(ret)?;
        Ok(())
    }
}

pub fn get_hardware_info(device_index: u8) -> Result<(String, String, String)> {
    let mut model_vec: [u8; 24] = [0; 24];
    let mut partno_vec: [u8; 8] = [0; 8];
    let mut version_vec: [u8; 8] = [0; 8];
    unsafe {
        let ret = MH_GetHardwareInfo(
            device_index.into(),
            model_vec.as_mut_ptr().cast::<i8>(),
            partno_vec.as_mut_ptr().cast::<i8>(),
            version_vec.as_mut_ptr().cast::<i8>(),
        );
        handle_error(ret)?;
        Ok((
            convert_into_string(&model_vec),
            convert_into_string(&partno_vec),
            convert_into_string(&version_vec),
        ))
    }
}

pub fn get_feature(device_index: u8) -> Result<i32> {
    let mut features = 0i32;
    unsafe {
        let ret = MH_GetFeatures(device_index.into(), &raw mut features);
        handle_error(ret)?;
        Ok(features)
    }
}

pub fn get_serial_number(device_index: u8) -> Result<String> {
    let mut vec_serial: [u8; 9] = [0; 9];
    unsafe {
        let ret = MH_GetSerialNumber(device_index.into(), vec_serial.as_mut_ptr().cast::<i8>());
        handle_error(ret)?;
        Ok(convert_into_string(&vec_serial))
    }
}

pub fn get_base_resolution(device_index: u8) -> Result<(f64, i32)> {
    let mut resolution: f64 = 0.0;
    let mut bin_steps: i32 = 0;
    unsafe {
        let ret =
            MH_GetBaseResolution(device_index.into(), &raw mut resolution, &raw mut bin_steps);
        handle_error(ret)?;
        Ok((resolution, bin_steps))
    }
}

pub fn get_number_of_input_channels(device_index: u8) -> Result<i32> {
    let mut num_channels: i32 = 0;
    unsafe {
        let ret = MH_GetNumOfInputChannels(device_index.into(), &raw mut num_channels);
        handle_error(ret)?;
        Ok(num_channels)
    }
}

pub fn get_number_of_modules(device_index: u8) -> Result<i32> {
    let mut number_of_modules = 0;
    unsafe {
        let ret = MH_GetNumOfModules(device_index.into(), &raw mut number_of_modules);
        handle_error(ret)?;
        Ok(number_of_modules)
    }
}

pub fn get_module_info(device_index: u8, module_index: u8) -> Result<(i32, i32)> {
    let mut model_code = 0i32;
    let mut version_code = 0i32;
    unsafe {
        let ret = MH_GetModuleInfo(
            device_index.into(),
            module_index.into(),
            &raw mut model_code,
            &raw mut version_code,
        );
        handle_error(ret)?;
        Ok((model_code, version_code))
    }
}

pub fn get_debug_info(device_index: u8) -> Result<String> {
    // This looks like a mistake in the MultiHarp API; typically text
    // bytes would be unsigned.
    let mut debug_info: Vec<i8> = vec![0i8; 65536];
    unsafe {
        let ret = MH_GetDebugInfo(device_index.into(), debug_info.as_mut_ptr());
        handle_error(ret)?;
        let unsigned_debug_info: Vec<u8> = debug_info.into_iter().map(i8::cast_unsigned).collect();
        Ok(convert_into_string(&unsigned_debug_info))
    }
}

pub fn set_sync_divider(device_index: u8, divider: i32) -> Result<()> {
    unsafe {
        let ret = MH_SetSyncDiv(device_index.into(), divider);
        handle_error(ret)?;
        Ok(())
    }
}

pub fn set_sync_edge_trigger(device_index: u8, trigger_level: i32, mac_edge: Edge) -> Result<()> {
    unsafe {
        let ret = MH_SetSyncEdgeTrg(device_index.into(), trigger_level, mac_edge as i32);
        handle_error(ret)?;
        Ok(())
    }
}

pub fn set_sync_channel_offset(device_index: u8, sync_timing_offset: i32) -> Result<()> {
    unsafe {
        let ret = MH_SetSyncChannelOffset(device_index.into(), sync_timing_offset);
        handle_error(ret)?;
        Ok(())
    }
}

pub fn set_sync_channel_enable(device_index: u8, enable: bool) -> Result<()> {
    unsafe {
        let ret = MH_SetSyncChannelEnable(device_index.into(), i32::from(enable));
        handle_error(ret)?;
        Ok(())
    }
}

pub fn set_sync_deadtime(device_index: u8, on: bool, deadtime_ps: i32) -> Result<()> {
    unsafe {
        let ret = MH_SetSyncDeadTime(device_index.into(), i32::from(on), deadtime_ps);
        handle_error(ret)?;
        Ok(())
    }
}

pub fn set_input_edge_trigger(
    device_index: u8,
    channel: MH160InternalChannelId,
    level: i32,
    mac_edge: Edge,
) -> Result<()> {
    let channel_u8: u8 = channel.into();
    unsafe {
        let ret = MH_SetInputEdgeTrg(
            device_index.into(),
            channel_u8.into(),
            level,
            mac_edge as i32,
        );
        handle_error(ret)?;
        Ok(())
    }
}

pub fn set_input_channel_offset(
    device_index: u8,
    channel: MH160InternalChannelId,
    offset: i32,
) -> Result<()> {
    let channel_u8: u8 = channel.into();
    unsafe {
        let ret = MH_SetInputChannelOffset(device_index.into(), channel_u8.into(), offset);
        handle_error(ret)?;
        Ok(())
    }
}

pub fn set_input_channel_enable(
    device_index: u8,
    channel: MH160InternalChannelId,
    enable: bool,
) -> Result<()> {
    let channel_u8: u8 = channel.into();
    unsafe {
        let ret =
            MH_SetInputChannelEnable(device_index.into(), channel_u8.into(), i32::from(enable));
        handle_error(ret)?;
        Ok(())
    }
}

pub fn set_input_deadtime(
    device_index: u8,
    channel: MH160InternalChannelId,
    on: bool,
    deadtime_ps: i32,
) -> Result<()> {
    let channel_u8: u8 = channel.into();
    unsafe {
        let ret = MH_SetInputDeadTime(
            device_index.into(),
            channel_u8.into(),
            i32::from(on),
            deadtime_ps,
        );
        handle_error(ret)?;
        Ok(())
    }
}

pub fn set_input_hysteresis(device_index: u8, hyst_code: u8) -> Result<()> {
    unsafe {
        let ret = MH_SetInputHysteresis(device_index.into(), hyst_code.into());
        handle_error(ret)?;
        Ok(())
    }
}

pub fn set_stop_overflow(device_index: u8, stop_overflow: bool, stop_count: u32) -> Result<()> {
    unsafe {
        let ret = MH_SetStopOverflow(device_index.into(), i32::from(stop_overflow), stop_count);
        handle_error(ret)?;
        Ok(())
    }
}

pub fn set_binning(device_index: u8, binning: i32) -> Result<()> {
    unsafe {
        let ret = MH_SetBinning(device_index.into(), binning);
        handle_error(ret)?;
        Ok(())
    }
}

pub fn set_offset(device_index: u8, offset: i32) -> Result<()> {
    unsafe {
        let ret = MH_SetOffset(device_index.into(), offset);
        handle_error(ret)?;
        Ok(())
    }
}

pub fn set_histogram_length(device_index: u8, len_code: i32) -> Result<i32> {
    let mut actual_length = 0i32;
    unsafe {
        let ret = MH_SetHistoLen(device_index.into(), len_code, &raw mut actual_length);
        handle_error(ret)?;
        Ok(actual_length)
    }
}

pub fn clear_histogram_memory(device_index: u8) -> Result<()> {
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
) -> Result<()> {
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

pub fn set_trigger_output(device_index: u8, period_100ns: i32) -> Result<()> {
    unsafe {
        let ret = MH_SetTriggerOutput(device_index.into(), period_100ns);
        handle_error(ret)?;
        Ok(())
    }
}

pub fn start_measurement(device_index: u8, acquisition_time: i32) -> Result<()> {
    unsafe {
        let ret = MH_StartMeas(device_index.into(), acquisition_time);
        handle_error(ret)?;
        Ok(())
    }
}

pub fn stop_measurement(device_index: u8) -> Result<()> {
    unsafe {
        let ret = MH_StopMeas(device_index.into());
        handle_error(ret)?;
        Ok(())
    }
}

pub fn ctc_status(device_index: u8) -> Result<i32> {
    let mut ctc_status_val = 0i32;
    unsafe {
        let ret = MH_CTCStatus(device_index.into(), &raw mut ctc_status_val);
        handle_error(ret)?;
        Ok(ctc_status_val)
    }
}

pub fn get_histogram(device_index: u8, channel: MH160InternalChannelId) -> Result<Vec<u32>> {
    let channel_u8: u8 = channel.into();
    let mut histogram_vec: Vec<u32> = vec![0u32; 65536];
    unsafe {
        let ret = MH_GetHistogram(
            device_index.into(),
            histogram_vec.as_mut_ptr(),
            channel_u8.into(),
        );
        handle_error(ret)?;
        Ok(histogram_vec)
    }
}

pub fn get_all_histogram(device_index: u8) -> Result<Vec<u32>> {
    let mut histogram_vec: Vec<u32> = vec![0u32; 65536];
    unsafe {
        let ret = MH_GetAllHistograms(device_index.into(), histogram_vec.as_mut_ptr());
        handle_error(ret)?;
        Ok(histogram_vec)
    }
}

pub fn get_resolution(device_index: u8) -> Result<f64> {
    let mut resolution: f64 = 0.0;
    unsafe {
        let ret = MH_GetResolution(device_index.into(), &raw mut resolution);
        handle_error(ret)?;
        Ok(resolution)
    }
}

pub fn get_sync_rate(device_index: u8) -> Result<i32> {
    let mut sync_rate: i32 = 0;
    unsafe {
        let ret = MH_GetSyncRate(device_index.into(), &raw mut sync_rate);
        handle_error(ret)?;
        Ok(sync_rate)
    }
}

pub fn get_count_rate(device_index: u8, channel: MH160InternalChannelId) -> Result<i32> {
    let mut count_rate: i32 = 0;
    let channel_u8: u8 = channel.into();
    unsafe {
        let ret = MH_GetCountRate(device_index.into(), channel_u8.into(), &raw mut count_rate);
        handle_error(ret)?;
        Ok(count_rate)
    }
}

pub fn get_all_count_rates(device_index: u8) -> Result<(i32, Vec<i32>)> {
    let mut sync_rate: i32 = 0;
    let mut count_rates = [0i32; MAXINPCHAN as usize];
    unsafe {
        let ret = MH_GetAllCountRates(
            device_index.into(),
            &raw mut sync_rate,
            count_rates.as_mut_ptr(),
        );
        handle_error(ret)?;
        Ok((sync_rate, count_rates.to_vec()))
    }
}

pub fn get_flags(device_index: u8) -> Result<i32> {
    let mut flags = 0i32;
    unsafe {
        let ret = MH_GetFlags(device_index.into(), &raw mut flags);
        handle_error(ret)?;
        Ok(flags)
    }
}

pub fn get_elapsed_measurement_time(device_index: u8) -> Result<f64> {
    let mut elapsed_time = 0f64;
    unsafe {
        let ret = MH_GetElapsedMeasTime(device_index.into(), &raw mut elapsed_time);
        handle_error(ret)?;
        Ok(elapsed_time)
    }
}

pub fn get_start_time(device_index: u8) -> Result<(u32, u32, u32)> {
    let mut time2 = 0u32;
    let mut time1 = 0u32;
    let mut time0 = 0u32;
    unsafe {
        let ret = MH_GetStartTime(
            device_index.into(),
            &raw mut time2,
            &raw mut time1,
            &raw mut time0,
        );
        handle_error(ret)?;
        Ok((time2, time1, time0))
    }
}

pub fn get_warnings(device_index: u8) -> Result<String> {
    let mut warnings: i32 = 0;
    let mut text = [0u8; 16384];
    unsafe {
        let ret = MH_GetWarnings(device_index.into(), &raw mut warnings);
        handle_error(ret)?;
        let ret = MH_GetWarningsText(
            device_index.into(),
            text.as_mut_ptr().cast::<i8>(),
            warnings,
        );
        handle_error(ret)?;
        Ok(convert_into_string(&text))
    }
}

pub fn read_fifo(device_index: u8) -> Result<Vec<u32>> {
    let mut num_records: i32 = 0;
    // TODO there should be a way to use Vec's spare_capacity_mut here
    // to avoid initialization overhead, but we would need to change
    // the C method signature to accept a MaybeUninit.
    let mut record_buffer: Vec<u32> = vec![0u32; mhlib_wrapper_header::TTREADMAX];
    unsafe {
        let ret = MH_ReadFiFo(
            device_index.into(),
            record_buffer.as_mut_ptr(),
            &raw mut num_records,
        );
        handle_error(ret)?;
        record_buffer.set_len(num_records.try_into()?);
        Ok(record_buffer)
    }
}

pub fn is_measurement_running(device_index: u8) -> Result<bool> {
    let mut ctc_status: i32 = 0;
    unsafe {
        let ret = MH_CTCStatus(device_index.into(), &raw mut ctc_status);
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
            open_device(0).unwrap_err().to_string(),
            String::from("MH_ERROR_DEVICE_OPEN_FAIL")
        );
    }
}
