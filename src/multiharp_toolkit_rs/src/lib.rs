
mod mhlib_wrapper;
// use pyo3::prelude::*;
// #[allow(non_upper_case_globals)]
// #[allow(non_camel_case_types)]
// #[allow(non_snake_case)]

// /// Formats the sum of two numbers as string.
// #[pyfunction]
// fn sum_as_string(a: usize, b: usize) -> PyResult<String> {
//     Ok((a + b).to_string())
// }

// /// A Python module implemented in Rust.
// #[pymodule]
// #[pyo3(name = "multiharp_toolkit_rs")]
// fn multiharp_toolkit(_py: Python, m: &PyModule) -> PyResult<()> {
//     m.add_function(wrap_pyfunction!(sum_as_string, m)?)?;
//     // m.add_function(wrap_pyfunction!(MH_GetLibraryVersion, m)?)?;
//     Ok(())
// }
