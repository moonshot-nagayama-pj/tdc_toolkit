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

pub fn initialize(device_index: u8, mode: Mode, ref_source: RefSource) -> Result<(), String> {
    unsafe {
        let ret = MH_Initialize(device_index.into(), mode as i32, ref_source as i32);
        handle_error(ret)?;
        Ok(())
    }
}

pub fn close_device(device_index: u8) -> Result<(), String> {
    unsafe {
        let ret = MH_CloseDevice(device_index.into());
        handle_error(ret)?;
        Ok(())
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

pub fn get_number_of_input_channels(device_index: u8) -> Result<i32, String> {
    let mut num_channels: i32 = 0;
    unsafe {
        let ret = MH_GetNumOfInputChannels(device_index.into(), &mut num_channels);
        handle_error(ret)?;
        Ok(num_channels)
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
