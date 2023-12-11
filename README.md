# MultiHarp Toolkit

# Getting started

prepare the environment.
if you don't have .venv dir, run it, or you can use other environment like anaconda
```sh
$ python -m venv .venv

# activate the venv. if you use vscode and its extension, it runs this automatically.
$ source .venv/bin/activate

# check
$ which python
# e.g. ~/multiharp-toolkit/.venv/bin/python
```

then we need to install the dependencies.
```sh
$ pip install -r requirements.lock
# if you want to use jupyterlab
$ pip install -r requirements-dev.lock
```

finally you can use `example.ipynb`
```sh
$ jupyter lab
```

