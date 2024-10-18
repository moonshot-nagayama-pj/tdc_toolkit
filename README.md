# MultiHarp Toolkit

## Prerequisites

* [rust](https://rustup.rs/)
* [rye](https://rye.astral.sh/)
* [shellcheck](https://www.shellcheck.net/)
* shfmt from the [sh](https://github.com/mvdan/sh/) tool collection

This library depends on a proprietary driver library from [PicoQuant](https://www.picoquant.com/) that is only available for the x64 architecture. Due to this library's license terms, we cannot distribute it with this library. Instead, it must be downloaded. This download will happen automatically the first time the Rust components of this project are built.

If you wish to develop this library on a non-x64 computer, such as an Apple Silicon or ARM device, it is necessary to emulate an x64 computer. For Apple Silicon users, the easiest way to accomplish this is to use a VSCode devcontainer; the configuration is available in this repository.

For performance reasons, production deployment is only recommended on native x64 devices.

## Getting started

This Python module calls PicoQuant's proprietary driver library via a Rust wrapper. The easiest way to build all code for development, including both Rust and Python, is to run the check script:

```sh
bin/check.bash
```

This will build the code and then run static analysis and unit tests. The same script runs on all pull requests, and must pass before a pull request is accepted.

Before using more specific commands such as `rye sync` or `maturin develop`, be sure to activate the virtual environment:

```
source .venv/bin/activate
```

The check script automatically enters the virtual environment while the script is running.

If you are using a devcontainer on Apple Silicon, you will also need to install the [`polars-lts-cpu`](https://pypi.org/project/polars-lts-cpu/) extension (see "Legacy" in the linked PyPI page).

```sh
$ python -m pip install polars-lts-cpu
```

The wrapper interface to the proprietary library is described in `python/multiharp_toolkit/_mhtk_rs.pyi`.
