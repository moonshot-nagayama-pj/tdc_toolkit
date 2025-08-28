# tdc_toolkit

Rust command-line interface and library, as well as Python bindings, for working with [time-to-digital converters (TDCs)](https://en.wikipedia.org/wiki/Time-to-digital_converter) such as the [PicoQuant MultiHarp 160](https://www.picoquant.com/products/category/tcspc-and-time-tagging-modules/multiharp-160).

## Availability

[Releases are available from crates.io](https://crates.io/crates/tdc_toolkit). Prebuilt binaries are not available at this time. See "Command-line interface," below, for installation and usage details.

## Supported devices

Only the PicoQuant MultiHarp 150/160 series of devices are supported at this time. Stub implementations of this device are also available for testing and development.

### PicoQuant MultiHarp

This driver was developed and tested using the [PicoQuant MultiHarp 160](https://www.picoquant.com/products/category/tcspc-and-time-tagging-modules/multiharp-160) on Linux `x86_64`.

Support depends on a proprietary driver library from [PicoQuant](https://www.picoquant.com/) that is only available for `x86_64` Linux and Windows. Due to the proprietary library's license terms, we cannot distribute it with `tdc_toolkit`. When building on Linux, the library will be automatically downloaded and linked when the `multiharp` feature is turned on; Windows requires manual installation.

When the `multiharp` feature is turned off, a stub implementation of the real driver is substituted. All other MultiHarp code remains present in the build. This allows MultiHarp-related development and testing on other architectures such as Apple Silicon/ARM.

## Command-line interface

### Install

Nearly all users will want MultiHarp support via the `multiharp` feature; see "Supported Devices," above, for details.

Issues and pull requests to expand the installation instructions are welcomed.

#### x86_64 Linux

```bash
# This will work on Debian and Ubuntu.
# Users of other distributions should install similar packages.
sudo apt install build-essential unzip

cargo install --features multiharp tdc_toolkit
```

The installation will take quite a long time, as it must download a large package of files from PicoQuant in order to obtain the proprietary driver library.

Users must also have appropriate permissions in order to access the MultiHarp device.

First, create a group, here called `multiharp`, and add your user to it. Then, create a `udev` rule with the following content to give that group access to the device. Place the following in `/etc/udev/rules.d/50-multiharp.rules`; this file should be owned by `root:root` and have permissions `644`.

```text
SUBSYSTEMS=="usb", ATTRS{idVendor}=="0e0d", ATTRS{idProduct}=="0013", GROUP="multiharp", MODE="0660"
```

After creating the group and the `udev` rule, log out, log back in, and physically unplug and re-plug the MultiHarp device.

It is also possible to forego `udev` configuration and instead run `tdc_toolkit` using `sudo`; this is not recommended.

#### x86_64 Windows

The installation process remains untested on Windows. However, this should work:

* [Download the PicoQuant proprietary library version 3.1](https://www.picoquant.com/dl_software/MultiHarp150/MultiHarp150_160_V3_1.zip); newer versions are not yet supported.
* Install the library at its default location: `C:\Program Files\PicoQuant\MultiHarp-MHLibv31`
* Follow [Microsoft's instructions for getting started with Rust development](https://learn.microsoft.com/en-us/windows/dev-environment/rust/setup).
* Run `cargo install --features multiharp tdc_toolkit`

#### Non-x86_64 Linux, MacOS, and other Unix-like operating systems

The `multiharp` feature can only be used on `x86_64` Linux and Windows. However, on other architectures and operating systems, the CLI can still be installed for evaluation.

You will need to find your distribution's equivalent to the Debian package `build-essential`. On MacOS, [installing the Xcode command line tools](https://developer.apple.com/xcode/resources/) should be sufficient.

```bash
# On Debian or Ubuntu
sudo apt install build-essential
cargo install tdc_toolkit
```

### Usage

Run `tdc_toolkit help` for full usage information.

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

The configuration file format is described in [the API documentation](https://docs.rs/tdc_toolkit/latest/tdc_toolkit/multiharp/device/struct.MH160DeviceConfig.html).

```bash
$ tdc_toolkit record --device-config=sample_config/multiharp160.json --duration 2s
[00:00:02] ████████████████████████████████████████ Recording complete
```

## Library

`tdc_toolkit` allows the construction of reusable, device-agnostic data processing pipelines. As an example, the MultiHarp's proprietary message format is first normalized into a common type before further processing occurs.

Consult the [the API documentation on docs.rs](https://docs.rs/tdc_toolkit/latest/tdc_toolkit/) for more information.

When enabling the `multiharp` feature, be sure to note the prerequisite requirements listed in the CLI section above.

## Python bindings

Basic typesafe Python bindings are available to control MultiHarp devices. They are documented in the `.pyi` files in `python/` and have not been extensively tested or released on PyPI. The easiest way to use them is as a [`uv` path dependency](https://docs.astral.sh/uv/concepts/projects/dependencies/#path). By default, the proprietary MultiHarp library is not used; the `features` key in the [`tool.maturin`] section of `pyproject.toml` must be modified to support it.

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
