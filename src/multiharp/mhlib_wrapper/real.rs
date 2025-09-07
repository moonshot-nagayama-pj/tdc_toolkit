use anyhow::{Result, anyhow};
use std::os::raw::c_int;
use libloading::Symbol;

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
    MH_SetRowEventFilter, MH_EnableRowEventFilter, MH_SetMainEventFilterParams, MH_SetMainEventFilterChannels,
    MH_EnableMainEventFilter, MH_SetFilterTestMode, MH_GetRowFilteredRates, MH_GetMainFilteredRates,
};

use super::meta;
use super::meta::{
    Edge, MH160InternalChannelId, MeasurementControl, MhlibWrapper, Mode, RefSource,
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

#[derive(PartialEq, Clone, Debug)]
pub struct MhlibWrapperReal {
    device_index: u8,
}

impl MhlibWrapperReal {
    #[must_use]
    pub fn new(device_index: u8) -> Self {
        Self { device_index }
    }
    fn assert_event_filter_supported(&self) -> Result<()> {
        let mut feat: i32 = 0;
        let rc = unsafe { MH_GetFeatures(self.device_index.into(), &mut feat) };
        handle_error(rc)?;
        const FEATURE_EVNT_FILT: i32 = 1 << 7;
        if (feat & FEATURE_EVNT_FILT) == 0 {
            anyhow::bail!("Event filtering not supported by this device/firmware");
        }
        Ok(())
    }
}

impl MhlibWrapper for MhlibWrapperReal {
    fn device_index(&self) -> u8 {
        self.device_index
    }

    fn get_library_version(&self) -> Result<String> {
        unsafe {
            let mut ver_str: [u8; 8] = [0; 8];
            let ret = MH_GetLibraryVersion(ver_str.as_mut_ptr().cast::<i8>());
            handle_error(ret)?;
            Ok(convert_into_string(&ver_str))
        }
    }

    fn open_device(&self) -> Result<String> {
        let mut vec_serial: [u8; 9] = [0; 9];
        unsafe {
            let ret = MH_OpenDevice(
                self.device_index.into(),
                vec_serial.as_mut_ptr().cast::<i8>(),
            );
            handle_error(ret)?;
            Ok(convert_into_string(&vec_serial))
        }
    }

    fn close_device(&self) -> Result<()> {
        unsafe {
            let ret = MH_CloseDevice(self.device_index.into());
            handle_error(ret)?;
            Ok(())
        }
    }

    fn initialize(&self, mode: Mode, ref_source: RefSource) -> Result<()> {
        unsafe {
            let ret = MH_Initialize(self.device_index.into(), mode as i32, ref_source as i32);
            handle_error(ret)?;
            Ok(())
        }
    }

    fn get_hardware_info(&self) -> Result<(String, String, String)> {
        let mut model_vec: [u8; 24] = [0; 24];
        let mut partno_vec: [u8; 8] = [0; 8];
        let mut version_vec: [u8; 8] = [0; 8];
        unsafe {
            let ret = MH_GetHardwareInfo(
                self.device_index.into(),
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

    fn get_feature(&self) -> Result<i32> {
        let mut features = 0i32;
        unsafe {
            let ret = MH_GetFeatures(self.device_index.into(), &raw mut features);
            handle_error(ret)?;
            Ok(features)
        }
    }

    fn get_serial_number(&self) -> Result<String> {
        let mut vec_serial: [u8; 9] = [0; 9];
        unsafe {
            let ret = MH_GetSerialNumber(
                self.device_index.into(),
                vec_serial.as_mut_ptr().cast::<i8>(),
            );
            handle_error(ret)?;
            Ok(convert_into_string(&vec_serial))
        }
    }

    fn get_base_resolution(&self) -> Result<(f64, i32)> {
        let mut resolution: f64 = 0.0;
        let mut bin_steps: i32 = 0;
        unsafe {
            let ret = MH_GetBaseResolution(
                self.device_index.into(),
                &raw mut resolution,
                &raw mut bin_steps,
            );
            handle_error(ret)?;
            Ok((resolution, bin_steps))
        }
    }

    fn get_number_of_input_channels(&self) -> Result<i32> {
        let mut num_channels: i32 = 0;
        unsafe {
            let ret = MH_GetNumOfInputChannels(self.device_index.into(), &raw mut num_channels);
            handle_error(ret)?;
            Ok(num_channels)
        }
    }

    fn get_number_of_modules(&self) -> Result<i32> {
        let mut number_of_modules = 0;
        unsafe {
            let ret = MH_GetNumOfModules(self.device_index.into(), &raw mut number_of_modules);
            handle_error(ret)?;
            Ok(number_of_modules)
        }
    }

    fn get_module_info(&self, module_index: u8) -> Result<(i32, i32)> {
        let mut model_code = 0i32;
        let mut version_code = 0i32;
        unsafe {
            let ret = MH_GetModuleInfo(
                self.device_index.into(),
                module_index.into(),
                &raw mut model_code,
                &raw mut version_code,
            );
            handle_error(ret)?;
            Ok((model_code, version_code))
        }
    }

    fn get_debug_info(&self) -> Result<String> {
        // This looks like a mistake in the MultiHarp API; typically text
        // bytes would be unsigned.
        let mut debug_info: Vec<i8> = vec![0i8; 65536];
        unsafe {
            let ret = MH_GetDebugInfo(self.device_index.into(), debug_info.as_mut_ptr());
            handle_error(ret)?;
            let unsigned_debug_info: Vec<u8> =
                debug_info.into_iter().map(i8::cast_unsigned).collect();
            Ok(convert_into_string(&unsigned_debug_info))
        }
    }

    fn set_sync_divider(&self, divider: i32) -> Result<()> {
        unsafe {
            let ret = MH_SetSyncDiv(self.device_index.into(), divider);
            handle_error(ret)?;
            Ok(())
        }
    }

    fn set_sync_edge_trigger(&self, trigger_level: i32, mac_edge: Edge) -> Result<()> {
        unsafe {
            let ret = MH_SetSyncEdgeTrg(self.device_index.into(), trigger_level, mac_edge as i32);
            handle_error(ret)?;
            Ok(())
        }
    }

    fn set_sync_channel_offset(&self, sync_timing_offset: i32) -> Result<()> {
        unsafe {
            let ret = MH_SetSyncChannelOffset(self.device_index.into(), sync_timing_offset);
            handle_error(ret)?;
            Ok(())
        }
    }

    fn set_sync_channel_enable(&self, enable: bool) -> Result<()> {
        unsafe {
            let ret = MH_SetSyncChannelEnable(self.device_index.into(), i32::from(enable));
            handle_error(ret)?;
            Ok(())
        }
    }

    fn set_sync_deadtime(&self, on: bool, deadtime_ps: i32) -> Result<()> {
        unsafe {
            let ret = MH_SetSyncDeadTime(self.device_index.into(), i32::from(on), deadtime_ps);
            handle_error(ret)?;
            Ok(())
        }
    }

    fn set_input_edge_trigger(
        &self,
        channel: MH160InternalChannelId,
        level: i32,
        mac_edge: Edge,
    ) -> Result<()> {
        unsafe {
            let ret = MH_SetInputEdgeTrg(
                self.device_index.into(),
                channel.into(),
                level,
                mac_edge as i32,
            );
            handle_error(ret)?;
            Ok(())
        }
    }

    fn set_input_channel_offset(&self, channel: MH160InternalChannelId, offset: i32) -> Result<()> {
        unsafe {
            let ret = MH_SetInputChannelOffset(self.device_index.into(), channel.into(), offset);
            handle_error(ret)?;
            Ok(())
        }
    }

    fn set_input_channel_enable(
        &self,
        channel: MH160InternalChannelId,
        enable: bool,
    ) -> Result<()> {
        unsafe {
            let ret = MH_SetInputChannelEnable(
                self.device_index.into(),
                channel.into(),
                i32::from(enable),
            );
            handle_error(ret)?;
            Ok(())
        }
    }

    fn set_input_deadtime(
        &self,
        channel: MH160InternalChannelId,
        on: bool,
        deadtime_ps: i32,
    ) -> Result<()> {
        unsafe {
            let ret = MH_SetInputDeadTime(
                self.device_index.into(),
                channel.into(),
                i32::from(on),
                deadtime_ps,
            );
            handle_error(ret)?;
            Ok(())
        }
    }

    fn set_input_hysteresis(&self, hyst_code: u8) -> Result<()> {
        unsafe {
            let ret = MH_SetInputHysteresis(self.device_index.into(), hyst_code.into());
            handle_error(ret)?;
            Ok(())
        }
    }

    fn set_stop_overflow(&self, stop_overflow: bool, stop_count: u32) -> Result<()> {
        unsafe {
            let ret = MH_SetStopOverflow(
                self.device_index.into(),
                i32::from(stop_overflow),
                stop_count,
            );
            handle_error(ret)?;
            Ok(())
        }
    }

    fn set_binning(&self, binning: i32) -> Result<()> {
        unsafe {
            let ret = MH_SetBinning(self.device_index.into(), binning);
            handle_error(ret)?;
            Ok(())
        }
    }

    fn set_offset(&self, offset: i32) -> Result<()> {
        unsafe {
            let ret = MH_SetOffset(self.device_index.into(), offset);
            handle_error(ret)?;
            Ok(())
        }
    }

    fn set_histogram_length(&self, len_code: i32) -> Result<i32> {
        let mut actual_length = 0i32;
        unsafe {
            let ret = MH_SetHistoLen(self.device_index.into(), len_code, &raw mut actual_length);
            handle_error(ret)?;
            Ok(actual_length)
        }
    }

    fn clear_histogram_memory(&self) -> Result<()> {
        unsafe {
            let ret = MH_ClearHistMem(self.device_index.into());
            handle_error(ret)?;
            Ok(())
        }
    }

    fn set_measurement_control(
        &self,
        meas_control: MeasurementControl,
        start_edge: Edge,
        stop_edge: Edge,
    ) -> Result<()> {
        unsafe {
            let ret = MH_SetMeasControl(
                self.device_index.into(),
                meas_control as i32,
                start_edge as i32,
                stop_edge as i32,
            );
            handle_error(ret)?;
            Ok(())
        }
    }

    fn set_trigger_output(&self, period_100ns: i32) -> Result<()> {
        unsafe {
            let ret = MH_SetTriggerOutput(self.device_index.into(), period_100ns);
            handle_error(ret)?;
            Ok(())
        }
    }

    fn start_measurement(&self, acquisition_time: i32) -> Result<()> {
        unsafe {
            let ret = MH_StartMeas(self.device_index.into(), acquisition_time);
            handle_error(ret)?;
            Ok(())
        }
    }

    fn stop_measurement(&self) -> Result<()> {
        unsafe {
            let ret = MH_StopMeas(self.device_index.into());
            handle_error(ret)?;
            Ok(())
        }
    }

    fn ctc_status(&self) -> Result<i32> {
        let mut ctc_status_val = 0i32;
        unsafe {
            let ret = MH_CTCStatus(self.device_index.into(), &raw mut ctc_status_val);
            handle_error(ret)?;
            Ok(ctc_status_val)
        }
    }

    fn get_histogram(&self, channel: MH160InternalChannelId) -> Result<Vec<u32>> {
        let mut histogram_vec: Vec<u32> = vec![0u32; 65536];
        unsafe {
            let ret = MH_GetHistogram(
                self.device_index.into(),
                histogram_vec.as_mut_ptr(),
                channel.into(),
            );
            handle_error(ret)?;
            Ok(histogram_vec)
        }
    }

    fn get_all_histogram(&self) -> Result<Vec<u32>> {
        let mut histogram_vec: Vec<u32> = vec![0u32; 65536];
        unsafe {
            let ret = MH_GetAllHistograms(self.device_index.into(), histogram_vec.as_mut_ptr());
            handle_error(ret)?;
            Ok(histogram_vec)
        }
    }

    fn get_resolution(&self) -> Result<f64> {
        let mut resolution: f64 = 0.0;
        unsafe {
            let ret = MH_GetResolution(self.device_index.into(), &raw mut resolution);
            handle_error(ret)?;
            Ok(resolution)
        }
    }

    fn get_sync_rate(&self) -> Result<i32> {
        let mut sync_rate: i32 = 0;
        unsafe {
            let ret = MH_GetSyncRate(self.device_index.into(), &raw mut sync_rate);
            handle_error(ret)?;
            Ok(sync_rate)
        }
    }

    fn get_count_rate(&self, channel: MH160InternalChannelId) -> Result<i32> {
        let mut count_rate: i32 = 0;
        unsafe {
            let ret = MH_GetCountRate(
                self.device_index.into(),
                channel.into(),
                &raw mut count_rate,
            );
            handle_error(ret)?;
            Ok(count_rate)
        }
    }

    fn get_all_count_rates(&self) -> Result<(i32, Vec<i32>)> {
        let mut sync_rate: i32 = 0;
        let mut count_rates = [0i32; MAXINPCHAN as usize];
        unsafe {
            let ret = MH_GetAllCountRates(
                self.device_index.into(),
                &raw mut sync_rate,
                count_rates.as_mut_ptr(),
            );
            handle_error(ret)?;
            Ok((sync_rate, count_rates.to_vec()))
        }
    }

    fn get_flags(&self) -> Result<i32> {
        let mut flags = 0i32;
        unsafe {
            let ret = MH_GetFlags(self.device_index.into(), &raw mut flags);
            handle_error(ret)?;
            Ok(flags)
        }
    }

    fn get_elapsed_measurement_time(&self) -> Result<f64> {
        let mut elapsed_time = 0f64;
        unsafe {
            let ret = MH_GetElapsedMeasTime(self.device_index.into(), &raw mut elapsed_time);
            handle_error(ret)?;
            Ok(elapsed_time)
        }
    }

    fn get_start_time(&self) -> Result<(u32, u32, u32)> {
        let mut time2 = 0u32;
        let mut time1 = 0u32;
        let mut time0 = 0u32;
        unsafe {
            let ret = MH_GetStartTime(
                self.device_index.into(),
                &raw mut time2,
                &raw mut time1,
                &raw mut time0,
            );
            handle_error(ret)?;
            Ok((time2, time1, time0))
        }
    }

    fn get_warnings(&self) -> Result<String> {
        let mut warnings: i32 = 0;
        let mut text = [0u8; 16384];
        unsafe {
            let ret = MH_GetWarnings(self.device_index.into(), &raw mut warnings);
            handle_error(ret)?;
            let ret = MH_GetWarningsText(
                self.device_index.into(),
                text.as_mut_ptr().cast::<i8>(),
                warnings,
            );
            handle_error(ret)?;
            Ok(convert_into_string(&text))
        }
    }

    fn read_fifo(&self) -> Result<Vec<u32>> {
        let mut num_records: i32 = 0;
        // TODO there should be a way to use Vec's spare_capacity_mut here
        // to avoid initialization overhead, but we would need to change
        // the C method signature to accept a MaybeUninit.
        let mut record_buffer: Vec<u32> = vec![0u32; meta::TTREADMAX];
        unsafe {
            let ret = MH_ReadFiFo(
                self.device_index.into(),
                record_buffer.as_mut_ptr(),
                &raw mut num_records,
            );
            handle_error(ret)?;
            record_buffer.set_len(num_records.try_into()?);
            Ok(record_buffer)
        }
    }

    fn is_measurement_running(&self) -> Result<bool> {
        let mut ctc_status: i32 = 0;
        unsafe {
            let ret = MH_CTCStatus(self.device_index.into(), &raw mut ctc_status);
            handle_error(ret)?;
            Ok(ctc_status == 0)
        }
    }

    fn set_row_event_filter(
        &self,
        rowidx: i32,
        timerange_ps: i32,
        matchcnt: i32,
        inverse: bool,
        usechannels_bits: i32,
        passchannels_bits: i32,
    ) -> Result<()> {
        self.assert_event_filter_supported().ok(); // 任意（無ければ削除可）
        let rc = unsafe {
            MH_SetRowEventFilter(
                self.device_index.into(),
                rowidx,
                timerange_ps,
                matchcnt,
                if inverse { 1 } else { 0 },
                usechannels_bits,
                passchannels_bits,
            )
        };
        handle_error(rc)
    }

    fn enable_row_event_filter(&self, rowidx: i32, enable: bool) -> Result<()> {
        let rc = unsafe {
            MH_EnableRowEventFilter(self.device_index.into(), rowidx, if enable { 1 } else { 0 })
        };
        handle_error(rc)
    }

    fn set_main_event_filter_params(
        &self,
        timerange_ps: i32,
        matchcnt: i32,
        inverse: bool,
    ) -> Result<()> {
        let rc = unsafe {
            MH_SetMainEventFilterParams(
                self.device_index.into(),
                timerange_ps,
                matchcnt,
                if inverse { 1 } else { 0 },
            )
        };
        handle_error(rc)
    }

    fn set_main_event_filter_channels(
        &self,
        rowidx: i32,
        usechannels_bits: i32,
        passchannels_bits: i32,
    ) -> Result<()> {
        let rc = unsafe {
            MH_SetMainEventFilterChannels(
                self.device_index.into(),
                rowidx,
                usechannels_bits,
                passchannels_bits,
            )
        };
        handle_error(rc)
    }

    fn enable_main_event_filter(&self, enable: bool) -> Result<()> {
        let rc = unsafe {
            MH_EnableMainEventFilter(self.device_index.into(), if enable { 1 } else { 0 })
        };
        handle_error(rc)
    }

    fn set_filter_test_mode(&self, test_mode: bool) -> Result<()> {
        let rc = unsafe {
            MH_SetFilterTestMode(self.device_index.into(), if test_mode { 1 } else { 0 })
        };
        handle_error(rc)
    }

    fn get_row_filtered_rates(&self) -> Result<(i32, Vec<i32>)> {
        let mut sync: i32 = 0;
        let mut rates = vec![0i32; unsafe { MAXINPCHAN as usize }];
        let rc = unsafe {
            MH_GetRowFilteredRates(self.device_index.into(), &mut sync, rates.as_mut_ptr())
        };
        handle_error(rc)?;
        Ok((sync, rates))
    }

    fn get_main_filtered_rates(&self) -> Result<(i32, Vec<i32>)> {
        let mut sync: i32 = 0;
        let mut rates = vec![0i32; unsafe { MAXINPCHAN as usize }];
        let rc = unsafe {
            MH_GetMainFilteredRates(self.device_index.into(), &mut sync, rates.as_mut_ptr())
        };
        handle_error(rc)?;
        Ok((sync, rates))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_lib_version() {
        let wrapper = MhlibWrapperReal::new(0);
        assert_eq!(wrapper.get_library_version().unwrap(), String::from("3.1"));
    }
    #[test]
    fn test_open_device() {
        let wrapper = MhlibWrapperReal::new(0);
        assert_eq!(
            wrapper.open_device().unwrap_err().to_string(),
            String::from("MH_ERROR_DEVICE_OPEN_FAIL")
        );
    }
}
