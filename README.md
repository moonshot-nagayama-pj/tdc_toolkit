# TDC Toolkit / tdctk

Rust CLI and library, as well as Python bindings, for working with [time-to-digital converters (TDCs)](https://en.wikipedia.org/wiki/Time-to-digital_converter) such as the [PicoQuant MultiHarp 160](https://www.picoquant.com/products/category/tcspc-and-time-tagging-modules/multiharp-160).


## Under rewrite

Please note that as of spring 2025, this project is being rewritten to move most logic into Rust and provide a generic interface for non-MultiHarp time-to-digital devices in the branch `api-refactor`.

## Prerequisites

Please see [our engineering documentation](https://github.com/moonshot-nagayama-pj/public-documents) for information on prerequisite development tools.

This library depends on a proprietary driver library from [PicoQuant](https://www.picoquant.com/) that is only available for the x64 architecture. Due to this library's license terms, we cannot distribute it with this library. Instead, it must be downloaded. This download will happen automatically the first time the Rust components of this project are built.

When working with this library on non-x64 architectures, PicoQuant's x64-only drivers will not be downloaded, and associated functionality will not be available.

## Getting started

The easiest way to build all code for development, including both Rust and Python, is to run the check script:

```sh
bin/check.bash
```

This will build the code and then run static analysis and unit tests. The same script runs on all pull requests, and must pass before a pull request is accepted.

If you are not using `direnv` or a similar tool, be sure to activate the virtual environment before using more specific commands such as `uv sync` or `maturin develop`:

```
source .venv/bin/activate
```

## Python bindings

Basic typesafe Python bindings are available to control MultiHarp devices, and closely track the Rust API. They are documented in the `.pyi` files in `python/` and have not been extensively tested.

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
