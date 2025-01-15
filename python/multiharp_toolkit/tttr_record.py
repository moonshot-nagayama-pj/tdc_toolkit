from dataclasses import dataclass, field
from multiharp_toolkit.exceptions import InvalidStateException
from multiharp_toolkit.interface import MultiharpDeviceChannel
from types import TracebackType
from typing import Self
from multiharp_toolkit.units import mhtk_ureg
from pint import Quantity

import asyncio
import contextlib
import struct


def split_raw_t2_record(raw_record: int) -> tuple[int, int, int]:
    special = (raw_record >> 31) & 0x01 # highest bit
    channel = (raw_record >> 25) & 0x3F # next six bits
    time_tag = raw_record & 0x1FFFFFF # the rest
    return (special, channel, time_tag)


@dataclass(frozen=True, kw_only=True)
class RawRecords():
    raw_data: bytes
    record_count: int


@dataclass(frozen=True, kw_only=True)
class T2Record():
    channel: MultiharpDeviceChannel
    time_tag: Quantity


@dataclass(kw_only=True)
class T2RecordQueueProcessor(contextlib.AbstractAsyncContextManager[None]):
    t2wraparound_v2: int = field(default=33554432) # in time-tag units, e.g. one unit = 5 picoseconds when resolution is 5
    overflow_correction: int = field(default=0) # in time-tag units, e.g. one unit = 5 picoseconds when resolution is 5
    resolution: int = field(default=5) # picoseconds
    closed: bool = field(default=False) # Track whether this object has already been closed

    input_queue: asyncio.Queue[RawRecords]
    output_queue: asyncio.Queue[T2Record]

    async def __aenter__(self) -> None:
        await self.open()

    async def __aexit__(
        self,
        exc_type: type[BaseException] | None,
        exc_val: BaseException | None,
        exc_tb: TracebackType | None,
    ) -> None:
        self.close(exc_type=exc_type, exc_val=exc_val, exc_tb=exc_tb)

    async def open(self) -> None:
        if self.closed:
            raise InvalidStateException()
        while True:
            try:
                await self.__process_raw_records()
            except asyncio.QueueShutDown:
                self.close()

    def close(
            self,
            exc_type: type[BaseException] | None = None,
            exc_val: BaseException | None = None,
            exc_tb: TracebackType | None = None,
    ) -> None:
        if self.closed:
            if exc_val:
                raise InvalidStateException("An exception attempted to close the queue processor, but it was already closed.", exc_val)
            raise InvalidStateException()
        try:
            self.output_queue.shutdown()
        finally:
            self.closed = True

    async def __process_raw_records(self) -> None:
        raw_records = await self.input_queue.get()

        for record_counter in range(raw_records.record_count):
            start_byte = record_counter * 4
            record_int = int.from_bytes(raw_records.raw_data[start_byte:start_byte+32], "little", signed=False)
            special, channel, time_tag = split_raw_t2_record(record_int)
            if not self.__process_special_records(special, channel, time_tag):
                continue
            await self.__process_normal_record(channel, time_tag)

        self.input_queue.task_done()

    async def __process_normal_record(self, channel: int, time_tag: int) -> None:
        true_time = self.overflow_correction + time_tag
        await self.output_queue.put(
            T2Record(
                channel=MultiharpDeviceChannel.from_linear(channel),
                time_tag=((true_time * self.resolution) * mhtk_ureg.picoseconds),
            )
        )

    def __process_special_records(self, special: int, channel: int, time_tag: int) -> bool:
        if special != 1:
            return False
        if channel == 0x3F:  # Overflow
            if time_tag == 0:  # old style overflow, shouldn't happen
                self.overflow_correction += self.t2wraparound_v2
            else:
                self.overflow_correction += self.t2wraparound_v2 * time_tag
        # Discard other special records for now
        return True
