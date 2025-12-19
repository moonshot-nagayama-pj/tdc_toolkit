use anyhow::Result;
use pyo3::prelude::*;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use crate::multiharp::device::MH160Device as WrappedMH160Device;
use crate::multiharp::device::{
    MH160, MH160DeviceConfig, MH160DeviceInfo, MH160DeviceInputChannelConfig,
    MH160DeviceSyncChannelConfig,
};
use crate::multiharp::device_stub::MH160Stub as WrappedMH160Stub;
use crate::multiharp::mhlib_wrapper::meta::Edge;

#[cfg(feature = "multiharp")]
use crate::multiharp::mhlib_wrapper::real::MhlibWrapperReal;

#[cfg(not(feature = "multiharp"))]
use crate::multiharp::mhlib_wrapper::stub::MhlibWrapperStub;

use crate::multiharp::recording::record_multiharp_to_parquet as wrapped_record_multiharp_to_parquet;

#[pyclass]
#[derive(Clone)]
pub struct MH160Stub {
    wrapped: Arc<WrappedMH160Stub>,
}

#[pymethods]
impl MH160Stub {
    #[new]
    pub fn new() -> Self {
        MH160Stub {
            wrapped: Arc::new(WrappedMH160Stub {}),
        }
    }

    fn device_info(&self) -> MH160DeviceInfo {
        self.wrapped.device_info()
    }
}

impl Default for MH160Stub {
    fn default() -> Self {
        Self::new()
    }
}

#[pyclass]
#[derive(Clone)]
pub struct MH160Device {
    #[cfg(feature = "multiharp")]
    pub wrapped: Arc<WrappedMH160Device<MhlibWrapperReal>>,

    #[cfg(not(feature = "multiharp"))]
    pub wrapped: Arc<WrappedMH160Device<MhlibWrapperStub>>,
}

#[pymethods]
impl MH160Device {
    #[staticmethod]
    pub fn from_config(device_index: u8, config: MH160DeviceConfig) -> Result<MH160Device> {
        #[cfg(feature = "multiharp")]
        let wrapped = Arc::new(WrappedMH160Device::from_config(
            MhlibWrapperReal::new(device_index),
            config,
        )?);
        #[cfg(not(feature = "multiharp"))]
        let wrapped = Arc::new(WrappedMH160Device::from_config(
            MhlibWrapperStub::new(device_index),
            config,
        )?);

        Ok(MH160Device { wrapped })
    }

    fn device_info(&self) -> MH160DeviceInfo {
        self.wrapped.device_info()
    }
}

#[pyfunction]
pub fn record_mh160stub_to_parquet(
    device: MH160Stub,
    output_dir: PathBuf,
    duration: Duration,
    name: &str,
) -> Result<()> {
    wrapped_record_multiharp_to_parquet(device.wrapped, output_dir, duration, name.to_owned())
}

#[pyfunction]
pub fn record_mh160device_to_parquet(
    device: MH160Device,
    output_dir: PathBuf,
    duration: Duration,
    name: &str,
) -> Result<()> {
    wrapped_record_multiharp_to_parquet(device.wrapped, output_dir, duration, name.to_owned())
}

#[pymodule]
fn tdc_toolkit(m: &Bound<'_, PyModule>) -> PyResult<()> {
    register_multiharp_module(m)?;
    Ok(())
}

fn register_multiharp_module(parent_module: &Bound<'_, PyModule>) -> PyResult<()> {
    let m = PyModule::new(parent_module.py(), "multiharp")?;

    m.add_class::<Edge>()?;
    m.add_class::<MH160Stub>()?;
    m.add_class::<MH160Device>()?;
    m.add_class::<MH160DeviceConfig>()?;
    m.add_class::<MH160DeviceInfo>()?;
    m.add_class::<MH160DeviceInputChannelConfig>()?;
    m.add_class::<MH160DeviceSyncChannelConfig>()?;
    m.add_function(wrap_pyfunction!(record_mh160stub_to_parquet, &m)?)?;
    m.add_function(wrap_pyfunction!(record_mh160device_to_parquet, &m)?)?;

    parent_module.add_submodule(&m)?;
    Ok(())
}
