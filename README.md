# MultiHarp Toolkit

# Prerequisites

- [rust](https://rustup.rs/)
- [rye](https://rye-up.com/guide/) (optional)

if you use vscode's devcontainer, you don't need it. 
every tools are installed automatically.

# Getting started

first of all we need to prepare the environment.
if you use [rye](https://rye-up.com/guide/), just sync it and then add pip
because [maturin](https://www.maturin.rs/) requires pip with the selected python.
```sh
$ rye sync
$ python -m ensurepip
```

if you're using devcontainer on Apple silicon,
```sh
$ python -m pip install polars-lts-cpu
```

if you don't use rye, create virtualenv and install dependencies.
```sh
$ python -m venv .venv

# activate the venv. if you use vscode and the python extension, it runs this automatically.
$ source .venv/bin/activate

# and install dependencies
$ pip install -r requirements-dev.lock
```

then confirm you are usuing the correct python interpreter.
```sh
$ which python
# e.g. ~/multiharp-toolkit/.venv/bin/python
```

then we need to install the dependencies.
```sh
$ pip install -r requirements.lock
# if you want to use jupyterlab
$ pip install -r requirements-dev.lock
```

# build

we use maturin to build native ext written Rust.
```sh
$ maturin develop
# or 
$ maturin build
```

then you can use it in python like this
```py
import multiharp_toolkit as mh
print(mh.get_library_version())
```

