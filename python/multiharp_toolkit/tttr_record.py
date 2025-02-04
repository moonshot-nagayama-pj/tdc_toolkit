import asyncio
import contextlib
import time
from dataclasses import dataclass, field
from types import TracebackType

import structlog
from multiharp_toolkit.exceptions import InvalidStateException
from multiharp_toolkit.interface import MultiharpDeviceChannel, RawRecords

log = structlog.get_logger()


def split_raw_t2_record(raw_record: int) -> tuple[int, int, int]:
    special = (raw_record >> 31) & 0x01  # highest bit
    channel = (raw_record >> 25) & 0x3F  # next six bits
    time_tag = raw_record & 0x1FFFFFF  # the rest
    return (special, channel, time_tag)


# A tuple representing a single T2 record.
#
# The first int is the channel ID. Channel 0 is the sync channel: raw
# records sent with the "special" bit set indicating that they contain
# a sync timetag are translated to channel 0 here. To add to the
# confusion, "normal" channels are represented in the raw records
# starting from 0; they are shifted by 1 here, so channel 0 in the raw
# record becomes channel 1 here. Given that the MultiHarp has no more
# than 64 channels, an 8-bit unsigned int should be sufficient to
# represent this value.
#
# The second int is the time tag, in picoseconds. A 64-bit unsigned
# int should be sufficient for experiments of up to a few months in
# length. Note that the raw value is not in picoseconds; it has been
# converted here.
#
# A tuple is used to avoid performance penalties that would be caused
# by creating a new object for each record.
T2Record = tuple[int, int]


@dataclass(kw_only=True)
class T2RecordQueueProcessor(contextlib.AbstractAsyncContextManager[None]):
    t2wraparound_v2: int = field(
        default=33554432
    )  # in time-tag units, e.g. one unit = 5 picoseconds when resolution is 5
    overflow_correction: int = field(
        default=0
    )  # in time-tag units, e.g. one unit = 5 picoseconds when resolution is 5
    resolution: int = field(default=5)  # picoseconds

    # Track whether this object has already been closed. This object
    # can only be used once.
    closed: bool = field(default=False, init=False)

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
            raise InvalidStateException("Attempted to re-open a closed object.")
        while True:
            try:
                raw_records = await self.input_queue.get()
                start_time = time.perf_counter()
                await self.__process_raw_records(raw_records)
                await log.adebug(
                    event="Raw record processing time (seconds)",
                    processing_time=time.perf_counter() - start_time,
                )
                self.input_queue.task_done()
            except asyncio.QueueShutDown:
                self.close()
                break

    # pylint: disable=W0613
    def close(
        self,
        exc_type: type[BaseException] | None = None,
        exc_val: BaseException | None = None,
        exc_tb: TracebackType | None = None,
    ) -> None:
        if exc_val:
            log.error("close() was called due to an exception.", exc_info=exc_val)
        if self.closed:
            if exc_val:
                raise InvalidStateException(
                    "An exception attempted to close the queue processor, but it was already closed."
                ) from exc_val
            raise InvalidStateException("Attempted to re-close a closed object.")
        try:
            self.output_queue.shutdown()
        finally:
            self.closed = True

    async def __process_raw_records(self, raw_records: RawRecords) -> None:
        for raw_record in raw_records:
            special, channel, time_tag = split_raw_t2_record(raw_record)
            if not await self.__process_special_records(special, channel, time_tag):
                await self.__process_normal_record(channel, time_tag)

    async def __process_normal_record(self, channel: int, time_tag: int) -> None:
        true_time = self.overflow_correction + time_tag
        await self.output_queue.put(((channel + 1), (true_time * self.resolution)))

    async def __process_special_records(
        self, special: int, channel: int, time_tag: int
    ) -> bool:
        if special != 1:
            return False
        if channel == 0x3F:  # Overflow
            if time_tag == 0:  # old style overflow, shouldn't happen
                self.overflow_correction += self.t2wraparound_v2
            else:
                self.overflow_correction += self.t2wraparound_v2 * time_tag
            return True
        if channel == 0:
            # Sync channel
            true_time = self.overflow_correction + time_tag
            await self.output_queue.put(
                (MultiharpDeviceChannel.CHAN_8_SYNC, (true_time * self.resolution))
            )
            return True
        # TODO Currently, this code discards external marker special records.
        #
        # Specifically, a channel between 1 and 15 inclusive indicates an external
        # marker; see the MultiHarp manual.
        return True
