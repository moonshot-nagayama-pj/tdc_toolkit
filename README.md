# tdc_toolkit

Rust command-line interface and library, as well as Python bindings, for working with [time-to-digital converters (TDCs)](https://en.wikipedia.org/wiki/Time-to-digital_converter) such as the [PicoQuant MultiHarp 160](https://www.picoquant.com/products/category/tcspc-and-time-tagging-modules/multiharp-160).

## Supported devices

The PicoQuant MultiHarp 150/160 series of devices are the only ones supported at this time. Stub implementations of this device are also available for testing and development.

### PicoQuant MultiHarp

This library was originally developed and tested using the [PicoQuant MultiHarp 160](https://www.picoquant.com/products/category/tcspc-and-time-tagging-modules/multiharp-160). Support is available on the `x86_64` only,

Support depends on a proprietary driver library from [PicoQuant](https://www.picoquant.com/) that is only available for the `x86_64` architecture. Due to the proprietary library's license terms, we cannot distribute it with `tdc_toolkit`. Instead, it will be automatically downloaded when the `multiharp` feature is turned on.

When the `multiharp` feature is turned off, all MultiHarp support code remains present, but is only linked to a stub implementation of the real driver. This means it is still possible to develop for the MultiHarp on other architectures such as Apple Silicon/ARM before deploying to an `x86_64` device controller.

## Command-line interface

### Install

Nearly all users will want MultiHarp support via the `multiharp` feature; see "Supported Devices," above, for details.

```bash
cargo install --features multiharp tdc_toolkit
```

### Usage

Full usage information is available by running `tdc_toolkit help`.

#### info

Queries the connected device for its current configuration, and prints the result in a device-specific JSON format.

```bash
$ tdc_toolkit info
{
  "device_index": 1,
  "library_version": "1.0",
  "model": "Base stub device",
  "partno": "one",
  "version": "2.0",
  "serial_number": "abcd1234",
  "base_resolution": 5.0,
  "binsteps": 1,
  "num_channels": 8
}
```

#### record

Opens the device, configures it according to the provided configuration file, and records to [Apache Parquet](https://parquet.apache.org/) files for the specified time duration.

To improve performance, long recordings will result in more than one Parquet file; at intervals of approximately 2 gigabytes, the current output file is closed and a new one opened. Most tools that support reading Parquet can treat a directory of many files as a single data source.

```bash
$ tdc_toolkit record --device-config=sample_config/multiharp160.json --duration 2s
[00:00:02] ████████████████████████████████████████ Recording complete
```

## Library

`tdc_toolkit` allows the construction of reusable, device-agnostic data processing pipelines. As an example, the MultiHarp's proprietary message format is first normalized into a common type before further processing occurs.

More information is available in the API documentation on [docs.rs](https://docs.rs/).

## Python bindings

Basic typesafe Python bindings are available to control MultiHarp devices, and closely track the Rust API. They are documented in the `.pyi` files in `python/` and have not been extensively tested or released on PyPI.

At the moment, submodules such as `tdc_toolkit.multiharp` are not available as subpackages. This means that the following will not work:

```python
# This will not work
from tdc_toolkit.multiharp import MH160Device
```

Instead, this style is necessary:

```python
from tdc_toolkit import multiharp

multiharp.MH160Device(...)
```

## Contributing

Please see [`CONTRIBUTING.md`](CONTRIBUTING.md).

## Getting help

If you have a specific question about how to use our software that is not answered by the documentation, please feel free to create a GitHub issue.

## API stability

We follow the [Semantic Versioning 2.0.0](https://semver.org/) standard, with [the usual Rust community modifications](https://doc.rust-lang.org/cargo/reference/manifest.html#the-version-field).

As the library only supports one type of device at present, it is likely that the APIs and data formats used will change as more devices are added and the problem space is better understood. For this reason, as of July 2025, we do not anticipate making a `1.x.x` release in the near future. Until a `1.x.x` release, we will make an effort to increase the minor version on significant breaking changes.

## Citing

If our software significantly contributed to your research, we ask that you cite it in your publications.

The best way to do so is by using the metadata in [our `CITATION.cff` file](CITATION.cff).

GitHub automatically generates APA and BibTeX-style citations from this file and makes them available from the "Cite this repository" link on the right-hand side. However, these citations do not include our DOI, making it difficult to locate our software. Please try to include either:

* The [concept DOI](https://zenodo.org/help/versioning), which points to the software as a whole: `10.5281/zenodo.16932740`.
    - This can also be found in `CITATION.cff`.
* A version-specific DOI, which can be found by first looking up [our concept DOI](https://doi.org/10.5281/zenodo.16932740) and then checking the list of available versions at Zenodo. A new DOI is generated for each release.
