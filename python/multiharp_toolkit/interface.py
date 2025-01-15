from abc import ABC, abstractmethod
from dataclasses import dataclass, field
from enum import Enum, IntFlag
from multiharp_toolkit.tttr_record import RawRecords
from pint import Quantity
from typing import Self

import asyncio
import contextlib
import enum
import typing

@dataclass(frozen=True, kw_only=True)
class MultiharpDeviceInfo():
    """Amalgamation of device-related information collected from
    several different API calls, for convenience.

    """
    # MH_GetLibraryVersion
    library_version: str

    # MH_GetHardwareInfo
    device_index: int
    model: str
    partno: str
    version: str

    # MH_GetSerialNumber
    serial_number: str

    # MH_GetBaseResolution
    base_resolution: float
    binsteps: int

    # MH_GetNumOfInputChannels
    num_channels: int

@enum.unique
class MultiharpDeviceIndex(int, Enum):
    DEV_0 = 0
    DEV_1 = 1
    DEV_2 = 2
    DEV_3 = 3
    DEV_4 = 4
    DEV_5 = 5
    DEV_6 = 6
    DEV_7 = 7

@enum.unique
class MultiharpDeviceChannel(IntFlag):
    CHAN_0_LEFTMOST = 0x0001
    CHAN_1 = 0x0002
    CHAN_2 = 0x0004
    CHAN_3 = 0x0008
    CHAN_4 = 0x0010
    CHAN_5 = 0x0020
    CHAN_6 = 0x0040
    CHAN_7_RIGHTMOST = 0x0080
    CHAN_8_SYNC = 0x0100 # only available when row index is 0

    @classmethod
    def from_linear(cls, channel: int) -> Self:
        if channel < 0 or channel > 8:
            raise ValueError("Channel must be between 0 and 8. See documentation for help identifying channels.")
        return cls(1 << channel)

@dataclass(frozen=True, kw_only=True)
class MultiharpMainEventFilterRow():
    use_channels: MultiharpDeviceChannel
    pass_channels: MultiharpDeviceChannel

@dataclass(frozen=True, kw_only=True)
class MultiharpMainEventFilter():
    time_range: Quantity
    match_count: int
    inverse: bool = field(default=False)
    rows: dict[int, MultiharpMainEventFilterRow]

@dataclass(frozen=True, kw_only=True)
class MultiharpDevice(ABC, contextlib.AbstractAsyncContextManager[None]):
    @abstractmethod
    def get_device_info(self) -> MultiharpDeviceInfo:
        pass

    @abstractmethod
    async def record_measurement_async(self, measurement_duration: Quantity, output: typing.BinaryIO) -> None:
        pass

    @abstractmethod
    def record_measurement_blocking(self, measurement_duration: Quantity, output: typing.BinaryIO) -> None:
        pass

    @abstractmethod
    def set_input_channel_offset(self, device_index: MultiharpDeviceIndex, channel: MultiharpDeviceChannel, offset: Quantity) -> None:
        pass

    @abstractmethod
    def set_main_event_filter(self, event_filter: dict[MultiharpDeviceIndex, MultiharpMainEventFilter]) -> None:
        pass

    @abstractmethod
    async def stream_measurement(self) -> asyncio.Queue[RawRecords]:
        pass
