use crate::multiharp::mhlib_wrapper::meta::MhlibWrapper;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct Version {
    pub tdc_toolkit_version: String,
    pub tdc_toolkit_git_ref: String,
    pub mhlib_version: String,
    pub rust_version: String,
}

#[must_use]
pub fn get_version() -> Version {
    Version {
        tdc_toolkit_version: env!("CARGO_PKG_VERSION").to_string(),
        tdc_toolkit_git_ref: env!("TDC_TOOLKIT_GIT_REF").to_string(),
        rust_version: env!("TDC_TOOLKIT_RUSTC_VERSION").to_string(),
        mhlib_version: get_mhlib_version_from_wrapper(),
    }
}

#[cfg(feature = "multiharp")]
fn get_mhlib_version_from_wrapper() -> String {
    if let Ok(wrapper) = crate::multiharp::mhlib_wrapper::real::MhlibWrapperReal::new(0) {
        wrapper
            .get_library_version()
            .unwrap_or_else(|_| "unknown".to_string())
    } else {
        "unknown".to_string()
    }
}

#[cfg(not(feature = "multiharp"))]
fn get_mhlib_version_from_wrapper() -> String {
    let wrapper = crate::multiharp::mhlib_wrapper::stub::MhlibWrapperStub::new(0);
    wrapper
        .get_library_version()
        .unwrap_or_else(|_| "unknown".to_string())
}
