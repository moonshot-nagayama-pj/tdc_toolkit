import asyncio
import struct
import time
from queue import Queue
from types import TracebackType
from typing import cast

import multiharp_toolkit._mhtk_rs as mh
from multiharp_toolkit.ptu_parser import parse_header
from multiharp_toolkit.util_types import (
    DeviceConfig,
    MeasEndMarker,
    MeasStartMarker,
    RawMeasDataSequence,
)

from abc import ABC, abstractmethod
from dataclasses import dataclass


@dataclass(frozen=True, kw_only=True)
class DeviceInputChannelConfig(TypedDict):
    edge_trigger_level: int
    edge_trigger: "mh.Edge"
    channel_offset: int
    enable: bool


@dataclass(frozen=True, kw_only=True)
class DeviceConfig(TypedDict):
    sync_divider: int
    sync_edge_trigger_level: int  # mV
    sync_edge: "mh.Edge"
    sync_channel_offset: int  # ps
    sync_channel_enable: bool
    inputs: list[DeviceInputChannelConfig]


@dataclass(frozen=True, kw_only=True)
class MultiharpDevice(ABC):
    device_index: int


class Device:
    device_index: int
    is_open: bool
    config: DeviceConfig
    queue: Queue[RawMeasDataSequence]

    test_enabled: bool
    test_ptu_file: str | None

    def __init__(self, index: int, config: DeviceConfig) -> None:
        self.device_index = index
        self.is_open = False
        self.config = config
        self.oflcorrection = 0
        self.queue = Queue()

        self.test_enabled = False
        self.test_ptu_file = None

    def __open(self) -> None:
        if self.is_open:
            return
        dev_id = self.device_index
        mh.open_device(dev_id)
        self.is_open = True
        mh.initialize(dev_id, mh.Mode.T2, mh.RefSource.InternalClock)
        self.configure()

    def configure(self, config: DeviceConfig | None = None) -> None:
        if config:
            self.config = config
        c = self.config
        dev_id = self.device_index

        num_inputs = mh.get_number_of_input_channels(dev_id)
        if num_inputs != len(self.config["inputs"]):
            print(
                "warning: num_inputs != len(config.inputs)",
                num_inputs,
                len(self.config["inputs"]),
            )

        mh.set_sync_divider(dev_id, c["sync_divider"])
        mh.set_sync_edge_trigger(dev_id, c["sync_edge_trigger_level"], c["sync_edge"])
        mh.set_sync_channel_offset(dev_id, 0)
        mh.set_sync_channel_enable(dev_id, c["sync_channel_enable"])
        for ch in range(0, num_inputs):
            ch_config = self.config["inputs"][ch]
            mh.set_input_edge_trigger(
                dev_id, ch, ch_config["edge_trigger_level"], ch_config["edge_trigger"]
            )
            mh.set_input_channel_offset(dev_id, ch, ch_config["channel_offset"])
            mh.set_input_channel_enable(dev_id, ch, ch_config["enable"])

    def close(self) -> None:
        mh.close_device(self.device_index)
        self.is_open = False

    def open(self) -> "Device":
        self.__open()
        return self

    def __enter__(self) -> None:
        self.__open()

    def __exit__(
        self,
        exc_type: type[BaseException] | None,
        exc_value: BaseException | None,
        exc_tb: TracebackType | None,
    ) -> None:
        self.close()

    def start_measurement(self, meas_time: int) -> None:
        if self.test_enabled:
            self.test_measurement(meas_time)
            return
        dev_id = self.device_index
        self.oflcorrection = 0
        self.queue.put_nowait([MeasStartMarker(self.config, meas_time)])
        mh.start_measurement(dev_id, meas_time)
        while True:
            flags = mh.get_flags(dev_id)
            if flags & 2:
                print("fifo overrun")
                mh.stop_measurement(dev_id)
                self.queue.put_nowait([MeasEndMarker()])

            # https://github.com/pylint-dev/pylint/issues/9354
            # pylint: disable-next=unpacking-non-sequence
            num_records, data = mh.read_fifo(dev_id)
            if num_records > 0:
                self.queue.put_nowait(cast(RawMeasDataSequence, data))
            else:
                status = mh.ctc_status(dev_id)
                if status > 0:
                    break

        mh.stop_measurement(dev_id)
        self.queue.put_nowait([MeasEndMarker()])

    async def start_measurement_async(self, meas_time: int) -> None:
        if self.test_enabled:
            self.test_measurement(meas_time)
            return
        dev_id = self.device_index
        self.oflcorrection = 0
        self.queue.put_nowait([MeasStartMarker(self.config, meas_time)])
        mh.start_measurement(dev_id, meas_time)
        while True:
            flags = mh.get_flags(dev_id)
            if flags & 2:
                print("fifo overrun")
                mh.stop_measurement(dev_id)
                self.queue.put_nowait([MeasEndMarker()])

            # https://github.com/pylint-dev/pylint/issues/9354
            # pylint: disable-next=unpacking-non-sequence
            num_records, data = mh.read_fifo(dev_id)
            if num_records > 0:
                self.queue.put_nowait(cast(RawMeasDataSequence, data))
            else:
                status = mh.ctc_status(dev_id)
                if status > 0:
                    break
            await asyncio.sleep(0)

        mh.stop_measurement(dev_id)
        self.queue.put_nowait([MeasEndMarker()])

    def test_measurement(self, meas_time: int) -> None:
        assert self.test_enabled
        assert self.test_ptu_file is not None
        with open(self.test_ptu_file, "rb") as f:
            headers = parse_header(f)
            assert headers is not None
            tag_names, tag_values = headers
            num_records = tag_values[tag_names.index("TTResult_NumberOfRecords")]
            self.queue.put_nowait([MeasStartMarker(self.config, meas_time)])
            arr = []
            for i in range(0, num_records):
                data = struct.unpack("<I", f.read(4))[0]
                arr.append(data)
                if i % 64 == 0:
                    self.queue.put_nowait(arr.copy())
                    arr = []
                    time.sleep(0.01)
            if len(arr) > 0:
                self.queue.put_nowait(arr.copy())
            self.queue.put_nowait([MeasEndMarker()])


def list_device_index() -> list[int]:
    available_devices = []
    for i in range(0, 8):
        try:
            mh.open_device(i)
            available_devices.append(i)
        # pylint: disable-next=broad-exception-caught,unused-variable
        except Exception as e:  # noqa: F841
            pass

    for i in range(0, 8):
        mh.close_device(i)
    return available_devices
