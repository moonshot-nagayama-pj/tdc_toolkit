# pylint: disable=W0613

import datetime
import os
from enum import Enum
from typing import Self

# pylint: disable=C0103
class Edge(Enum):
    Falling = 0  # EDGE_FALLING
    Rising = 1  # EDGE_RISING

class MH160DeviceInfo:
    device_index: int
    library_version: str
    model: str
    partno: str
    version: str
    serial_number: str
    base_resolution: str
    binsteps: int
    num_channels: int

class MH160DeviceInputChannelConfig:
    id: int
    edge_trigger_level: int  # mV
    edge_trigger: Edge
    offset: int  # picoseconds

class MH160DeviceSyncChannelConfig:
    divider: int
    edge_trigger_level: int  # mV
    edge_trigger: Edge
    offset: int  # picoseconds

class MH160DeviceConfig:
    sync_channel: MH160DeviceSyncChannelConfig
    input_channels: list[MH160DeviceInputChannelConfig]

class MH160Stub:
    @classmethod
    def new(cls) -> Self: ...
    def get_device_info(self) -> MH160DeviceInfo: ...

class MH160Device:
    @classmethod
    def from_config(cls, device_index: int, config: MH160DeviceConfig) -> Self: ...
    def get_device_info(self) -> MH160DeviceInfo: ...

def record_mh160stub_to_parquet(
    device: MH160Stub,
    output_dir: os.PathLike[str],
    duration: datetime.timedelta,
    name: str,
) -> None: ...
def record_mh160device_to_parquet(
    device: MH160Stub,
    output_dir: os.PathLike[str],
    duration: datetime.timedelta,
    name: str,
) -> None: ...
