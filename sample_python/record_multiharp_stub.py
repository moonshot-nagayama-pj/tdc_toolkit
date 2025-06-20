#!/usr/bin/env python

import datetime
from pathlib import PurePath
from tempfile import TemporaryDirectory

from tdc_toolkit import multiharp


def main() -> None:
    stub = multiharp.MH160Stub()
    print(stub.get_device_info())

    duration = datetime.timedelta(seconds=1)
    with TemporaryDirectory(delete=False) as temp_dir:
        print(f"Files written to temporary directory {temp_dir}")
        temp_dir_path = PurePath(temp_dir)
        multiharp.record_mh160stub_to_parquet(stub, temp_dir_path, duration, "example")


if __name__ == "__main__":
    main()
