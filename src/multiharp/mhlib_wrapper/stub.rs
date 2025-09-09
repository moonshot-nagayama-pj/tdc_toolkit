use anyhow::Result;

use super::meta::{
    Edge, MH160InternalChannelId, MeasurementControl, MhlibWrapper, Mode, RefSource,
};

#[derive(PartialEq, Clone, Debug)]
pub struct MhlibWrapperStub {
    device_index: u8,
}

impl MhlibWrapperStub {
    #[must_use]
    pub fn new(device_index: u8) -> Self {
        Self { device_index }
    }
}

impl MhlibWrapper for MhlibWrapperStub {
    fn device_index(&self) -> u8 {
        self.device_index
    }

    fn get_library_version(&self) -> Result<String> {
        Ok("Stub".to_string())
    }

    fn open_device(&self) -> Result<String> {
        Ok("ABC123".to_string())
    }

    fn close_device(&self) -> Result<()> {
        Ok(())
    }

    fn initialize(&self, _mode: Mode, _ref_source: RefSource) -> Result<()> {
        Ok(())
    }

    fn get_hardware_info(&self) -> Result<(String, String, String)> {
        Ok((
            "model".to_string(),
            "part_no".to_string(),
            "version".to_string(),
        ))
    }

    fn get_feature(&self) -> Result<i32> {
        Ok(0i32)
    }

    fn get_serial_number(&self) -> Result<String> {
        Ok("ABC123".to_string())
    }

    fn get_base_resolution(&self) -> Result<(f64, i32)> {
        Ok((5.0_f64, 0_i32))
    }

    fn get_number_of_input_channels(&self) -> Result<i32> {
        Ok(8_i32)
    }

    fn get_number_of_modules(&self) -> Result<i32> {
        Ok(3_i32)
    }

    fn get_module_info(&self, _module_index: u8) -> Result<(i32, i32)> {
        Ok((0i32, 0i32))
    }

    fn get_debug_info(&self) -> Result<String> {
        Ok("debug_info".to_string())
    }

    fn set_sync_divider(&self, _divider: i32) -> Result<()> {
        Ok(())
    }

    fn set_sync_edge_trigger(&self, _trigger_level: i32, _mac_edge: Edge) -> Result<()> {
        Ok(())
    }

    fn set_sync_channel_offset(&self, _sync_timing_offset: i32) -> Result<()> {
        Ok(())
    }

    fn set_sync_channel_enable(&self, _enable: bool) -> Result<()> {
        Ok(())
    }

    fn set_sync_deadtime(&self, _on: bool, _deadtime_ps: i32) -> Result<()> {
        Ok(())
    }

    fn set_input_edge_trigger(
        &self,
        _channel: MH160InternalChannelId,
        _level: i32,
        _mac_edge: Edge,
    ) -> Result<()> {
        Ok(())
    }

    fn set_input_channel_offset(
        &self,
        _channel: MH160InternalChannelId,
        _offset: i32,
    ) -> Result<()> {
        Ok(())
    }

    fn set_input_channel_enable(
        &self,
        _channel: MH160InternalChannelId,
        _enable: bool,
    ) -> Result<()> {
        Ok(())
    }

    fn set_input_deadtime(
        &self,
        _channel: MH160InternalChannelId,
        _on: bool,
        _deadtime_ps: i32,
    ) -> Result<()> {
        Ok(())
    }

    fn set_input_hysteresis(&self, _hyst_code: u8) -> Result<()> {
        Ok(())
    }

    fn set_stop_overflow(&self, _stop_overflow: bool, _stop_count: u32) -> Result<()> {
        Ok(())
    }

    fn set_binning(&self, _binning: i32) -> Result<()> {
        Ok(())
    }

    fn set_offset(&self, _offset: i32) -> Result<()> {
        Ok(())
    }

    fn set_histogram_length(&self, _len_code: i32) -> Result<i32> {
        Ok(5i32)
    }

    fn clear_histogram_memory(&self) -> Result<()> {
        Ok(())
    }

    fn set_measurement_control(
        &self,
        _meas_control: MeasurementControl,
        _start_edge: Edge,
        _stop_edge: Edge,
    ) -> Result<()> {
        Ok(())
    }

    fn set_trigger_output(&self, _period_100ns: i32) -> Result<()> {
        Ok(())
    }

    fn start_measurement(&self, _acquisition_time: i32) -> Result<()> {
        Ok(())
    }

    fn stop_measurement(&self) -> Result<()> {
        Ok(())
    }

    fn ctc_status(&self) -> Result<i32> {
        Ok(0i32)
    }

    fn get_histogram(&self, _channel: MH160InternalChannelId) -> Result<Vec<u32>> {
        let histogram_vec: Vec<u32> = vec![0u32; 65536];
        Ok(histogram_vec)
    }

    fn get_all_histogram(&self) -> Result<Vec<u32>> {
        let histogram_vec: Vec<u32> = vec![0u32; 65536];
        Ok(histogram_vec)
    }

    fn get_resolution(&self) -> Result<f64> {
        Ok(0.0f64)
    }

    fn get_sync_rate(&self) -> Result<i32> {
        Ok(0i32)
    }

    fn get_count_rate(&self, _channel: MH160InternalChannelId) -> Result<i32> {
        Ok(0i32)
    }

    fn get_all_count_rates(&self) -> Result<(i32, Vec<i32>)> {
        let count_rates = [0i32; 64];
        Ok((0i32, count_rates.to_vec()))
    }

    fn get_flags(&self) -> Result<i32> {
        Ok(0i32)
    }

    fn get_elapsed_measurement_time(&self) -> Result<f64> {
        Ok(0f64)
    }

    fn get_start_time(&self) -> Result<(u32, u32, u32)> {
        Ok((0u32, 0u32, 0u32))
    }

    fn get_warnings(&self) -> Result<String> {
        Ok("warning".to_string())
    }

    fn read_fifo(&self) -> Result<Vec<u32>> {
        Ok(vec![0u32])
    }

    fn is_measurement_running(&self) -> Result<bool> {
        Ok(true)
    }

    fn set_row_event_filter(
        &self,
        _rowidx: i32,
        _time_range_ps: i32,
        _match_count: i32,
        _inverse: bool,
        _use_channels_bits: i32,
        _pass_channels_bits: i32,
    ) -> Result<()> {
        Ok(())
    }

    fn enable_row_event_filter(
        &self,
        _rowidx: i32,
        _enable: bool,
    ) -> Result<()> {
        Ok(())
    }

    fn set_main_event_filter_params(
        &self,
        _time_range_ps: i32,
        _match_count: i32,
        _inverse: bool,
    ) -> Result<()> {
        Ok(())
    }

    fn set_main_event_filter_channels(
        &self,
        _rowidx: i32,
        _use_channels_bits: i32,
        _pass_channels_bits: i32,
    ) -> Result<()> {
        Ok(())
    }

    fn enable_main_event_filter(
        &self,
        _enable: bool,
    ) -> Result<()> {
        Ok(())
    }

    fn set_filter_test_mode(
        &self,
        _test_mode: bool,
    ) -> Result<()> {
        Ok(())
    }

    fn get_row_filtered_rates(
        &self,
        num_channels: usize,
    ) -> Result<(i32, Vec<i32>)> {
        Ok((0, vec![0; num_channels]))
    }

    fn get_main_filtered_rates(
        &self,
        num_channels: usize,
    ) -> Result<(i32, Vec<i32>)> {
        Ok((0, vec![0; num_channels]))
    }
}
