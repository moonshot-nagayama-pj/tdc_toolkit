use crate::parser;
use parser::{parse_t2_ptu, PQTimeTaggedData};
use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;

#[pyfunction]
pub fn parse_rs(input: &[u8]) -> PyResult<PQTimeTaggedData> {
    match parse_t2_ptu(input) {
        Ok((_rest, data)) => Ok(data),
        _ => Err(PyRuntimeError::new_err("failed to parse")),
    }
}
