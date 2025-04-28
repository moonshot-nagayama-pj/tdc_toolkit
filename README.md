# TDC Toolkit / tdctk

Rust library and Python bindings for working with [time-to-digital converters (TDCs)](https://en.wikipedia.org/wiki/Time-to-digital_converter) such as the [PicoQuant MultiHarp 160](https://www.picoquant.com/products/category/tcspc-and-time-tagging-modules/multiharp-160).

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

If you are using a devcontainer on Apple Silicon, you will also need to install the [`polars-lts-cpu`](https://pypi.org/project/polars-lts-cpu/) extension (see "Legacy" in the linked PyPI page).

```sh
$ python -m pip install polars-lts-cpu
```

The wrapper interface to the proprietary library is described in `python/tdc_toolkit/tdctk_rs.pyi`.
