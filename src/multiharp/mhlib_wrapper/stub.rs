use anyhow::Result;
use std::sync::Mutex;
use std::thread;
use std::time::{Duration, Instant};

use super::meta::event_filter::{Inverse, MainEnabled, RowEnabled, TestMode};
use super::meta::{
    Edge, Features, FilteredRates, MAX_INPUT_CHANNEL, MH160InternalChannelId, MeasurementControl,
    MhlibWrapper, Mode, RefSource,
};

#[derive(Debug)]
pub struct MhlibWrapperStub {
    device_index: u8,
    measurement_end: Mutex<Option<Instant>>,
}

/// A stub implementation of the `MhlibWrapper` trait for testing purposes.
/// It simulates the behavior of the real MHLib wrapper without requiring
/// actual hardware interaction.
impl PartialEq for MhlibWrapperStub {
    fn eq(&self, other: &Self) -> bool {
        self.device_index == other.device_index
    }
}

/// The `Clone` implementation allows for creating a new instance of
/// `MhlibWrapperStub` with the same `device_index`, but with a new
/// `measurement_end` mutex initialized to `None`.
impl Clone for MhlibWrapperStub {
    fn clone(&self) -> Self {
        Self::new(self.device_index)
    }
}

impl MhlibWrapperStub {
    #[must_use]
    pub fn new(device_index: u8) -> Self {
        Self {
            device_index,
            measurement_end: Mutex::new(None),
        }
    }

    /// Returns `true` if the acquisition time set by `start_measurement` has elapsed (measurement complete).
    /// Returns `false` if no measurement has been started or the time has not yet elapsed (keeps the lock on `measurement_end`).
    fn measurement_is_complete(&self) -> bool {
        self.measurement_end
            // Locks the `measurement_end` mutex to safely access the end time. If the lock is poisoned, it panics.
            .lock()
            // If `measurement_end` = `None`, no measurement is running => 'false'
            .unwrap()
            // If `measurement_end` = `Some(end)`, return whether current time has passed `end`
            .is_some_and(|end| Instant::now() >= end)
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

    fn get_features(&self) -> Result<Features> {
        Ok(Features::EVNT_FILT)
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

    fn start_measurement(&self, acquisition_time: i32) -> Result<()> {
        // Sets the measurement end time to the current time plus the acquisition time,
        // allowing `measurement_is_complete` to return `true` once the acquisition time has elapsed.
        let end = Instant::now() + Duration::from_millis(acquisition_time.try_into()?);
        *self.measurement_end.lock().unwrap() = Some(end);
        Ok(())
    }

    /// Signals no measurement is running which frees the lock on the measurement end time
    /// and allows `measurement_is_complete` to return `false` until a new measurement
    /// is started and sets a new end time
    fn stop_measurement(&self) -> Result<()> {
        *self.measurement_end.lock().unwrap() = None;
        Ok(())
    }

    /// Mirrors the real MHLib `MH_CTCStatus` return value convention:
    /// `0` means the acquisition time has not yet elapsed (measurement still running),
    /// non-zero means the acquisition time has elapsed and the measurement is complete.
    fn ctc_status(&self) -> Result<i32> {
        Ok(i32::from(self.measurement_is_complete()))
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

    /// Returns stub FIFO data while the measurement is still running, and an empty
    /// buffer once the acquisition time has elapsed, signalling to the caller that
    /// there is no more data to read.
    ///
    /// Sleeps briefly between calls to simulate the real hardware's FIFO fill rate,
    /// preventing the polling loop in `do_stream_measurement` from spinning at full
    /// CPU speed and flooding downstream channels with an unbounded backlog.
    fn read_fifo(&self) -> Result<Vec<u32>> {
        if self.measurement_is_complete() {
            Ok(vec![])
        } else {
            thread::sleep(Duration::from_millis(100)); // Sleep justified in function docstring
            Ok(vec![0u32])
        }
    }

    fn is_measurement_running(&self) -> Result<bool> {
        Ok(!self.measurement_is_complete())
    }

    fn set_row_event_filter(
        &self,
        _rowidx: i32,
        _time_range_ps: i32,
        _match_count: i32,
        _inverse: Inverse,
        _use_channels_bits: i32,
        _pass_channels_bits: i32,
    ) -> Result<()> {
        Ok(())
    }

    fn enable_row_event_filter(&self, _rowidx: i32, _enable: RowEnabled) -> Result<()> {
        Ok(())
    }

    fn set_main_event_filter_params(
        &self,
        _time_range_ps: i32,
        _match_count: i32,
        _inverse: Inverse,
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

    fn enable_main_event_filter(&self, _enable: MainEnabled) -> Result<()> {
        Ok(())
    }

    fn set_filter_test_mode(&self, _test_mode: TestMode) -> Result<()> {
        Ok(())
    }

    fn get_row_filtered_rates(&self) -> Result<FilteredRates> {
        Ok(FilteredRates {
            sync_rate: 0,
            count_rates: vec![0; MAX_INPUT_CHANNEL as usize],
        })
    }

    fn get_main_filtered_rates(&self) -> Result<FilteredRates> {
        Ok(FilteredRates {
            sync_rate: 0,
            count_rates: vec![0; MAX_INPUT_CHANNEL as usize],
        })
    }
}
