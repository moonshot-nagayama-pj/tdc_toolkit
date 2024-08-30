import os
import time
from multiharp_toolkit.device import DeviceConfig
import asyncio
import pyarrow as pa
from multiharp_toolkit.util_types import (
    T2WRAPAROUND_V2,
    MeasEndMarker,
    MeasStartMarker,
    ParsedMeasDataSequence,
    RawMeasDataSequence,
    TimeTagDataSchema,
)

from queue import Empty, Queue


class StreamParser:
    oflcorrection: int
    config: DeviceConfig | None
    queue_send: asyncio.Queue[ParsedMeasDataSequence]
    """queue for sending the data to next step"""

    queue_recv: Queue[RawMeasDataSequence]
    """queue for receiving the raw measurement data"""

    writer: pa.RecordBatchFileWriter
    filename: str

    def __init__(
        self, queue_recv: Queue[RawMeasDataSequence], single_file: bool = True
    ) -> None:
        self.queue_send = asyncio.Queue()
        self.queue_recv = queue_recv
        self.oflcorrection = 0
        self.config = None
        self.single_file = single_file
        self.time_resolution = 5

    async def run(self):
        while True:
            try:
                data = self.queue_recv.get_nowait()
            except Empty:
                await asyncio.sleep(0)
                continue
            if not data:
                self.queue_recv.task_done()
                continue
            ch_arr = []
            ts_arr = []
            for val in data:
                if isinstance(val, MeasStartMarker):
                    self.create_file(val)
                    self.queue_send.put_nowait(val)
                elif isinstance(val, MeasEndMarker):
                    self.close_file(val)
                    self.queue_send.put_nowait(val)
                    if self.single_file:
                        return
                else:
                    special = (val >> 31) & 0x01  # 最上位ビット
                    channel = (val >> 25) & 0x3F  # 次の6ビット
                    timetag = val & 0x1FFFFFF
                    if special == 1:
                        if channel == 0x3F:  # Overflow
                            if timetag == 0:  # old style overflow, shouldn't happen
                                self.oflcorrection += T2WRAPAROUND_V2
                            else:
                                self.oflcorrection += T2WRAPAROUND_V2 * timetag
                        if channel == 0:  # sync
                            truetime = self.oflcorrection + timetag
                            ch_arr.append(channel)
                            ts_arr.append(self.convert_timetag_to_relative_timestamp(truetime))
                    else:  # regular input channel
                        truetime = self.oflcorrection + timetag
                        ch_arr.append(channel + 1)
                        ts_arr.append(self.convert_timetag_to_relative_timestamp(truetime))
            if ch_arr:
                batch = pa.record_batch(
                    [
                        pa.array(ch_arr, type=pa.int8()),
                        pa.array(ts_arr, type=pa.float64()),
                    ],
                    names=["ch", "timestamp"],
                )
                self.queue_send.put_nowait(batch)
                self.writer.write_batch(batch)
                ch_arr = []
                ts_arr = []
            self.queue_recv.task_done()

    def convert_timetag_to_relative_timestamp(self, timetag: int) -> int:
        """Convert a time tag to a relative timestamp with picosecond resolution.
        """
        return timetag * self.time_resolution

    def create_file(self, marker: MeasStartMarker):
        self.oflcorrection = 0
        os.mkdirs(".arrows", 0o755, exist_ok=True)
        filename = os.path.join(
            ".arrows", f"{int(time.time())}-{marker.measurement_duration}.arrow"
        )
        self.writer = pa.ipc.new_file(
            filename, schema=TimeTagDataSchema.with_metadata({"ch": str(marker.config)})
        )
        self.filename = filename
        print("open file: ", filename)

    def close_file(self, marker: MeasEndMarker):
        self.writer.close()
        print("close file ")
