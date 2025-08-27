//! Rust command-line interface and library, as well as Python bindings, for working with [time-to-digital converters (TDCs)](https://en.wikipedia.org/wiki/Time-to-digital_converter) such as the [PicoQuant MultiHarp 160](https://www.picoquant.com/products/category/tcspc-and-time-tagging-modules/multiharp-160).
//!
//! Please see the [project README](https://github.com/moonshot-nagayama-pj/tdc_toolkit) for CLI usage information.
//!
//! ## How to use
//!
//! This library contains components that can be connected through [standard MPSC Rust channels](std::sync::mpsc) to form data processing pipelines. Broadly speaking, these components fall into the folowing categories:
//!
//! ### Device
//!
//! A pipeline usually starts with a device. These may be real, physical devices such as the MultiHarp 160, or they may be simulated devices. The goal of the device component is to produce data that closely follows the proprietary format of the device.
//!
//! * [multiharp]
//!
//! ### Normalizer
//!
//! After the device produces a stream of proprietary data, the normalizer transforms it into a common format that can be used for further processing.
//!
//! * [multiharp]
//!
//! ### Output
//!
//! Output components accept normalized data and do something with it, like write it to disk.
//!
//! * [output]
//!
//! ## Examples
//!
//! For now, the best example of a complete pipeline is probably the `tdc_toolkit` CLI implementation.

pub mod multiharp;
pub mod output;
pub mod types;

#[cfg(feature = "python")]
mod python_api;
