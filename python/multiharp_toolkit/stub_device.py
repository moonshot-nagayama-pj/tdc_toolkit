import asyncio
import time
from dataclasses import dataclass

from multiharp_toolkit.interface import (
    MultiharpDevice,
    MultiharpDeviceChannel,
    MultiharpDeviceIndex,
    MultiharpDeviceInfo,
    MultiharpMainEventFilter,
    RawRecords,
)
from multiharp_toolkit.units import mhtk_ureg
from pint import Quantity


@dataclass(frozen=True, kw_only=True)
class StubMultiharpDevice(MultiharpDevice):
    def get_device_info(self) -> MultiharpDeviceInfo:
        return MultiharpDeviceInfo(
            library_version="1.0",
            device_index=1,
            model="Base stub device",
            partno="one",
            version="2.0",
            serial_number="abcd1234",
            base_resolution=5.0,
            binsteps=1,
            num_channels=8,
        )

    def set_input_channel_offset(
        self,
        device_index: MultiharpDeviceIndex,
        channel: MultiharpDeviceChannel,
        offset: Quantity,
    ) -> None:
        pass

    def set_main_event_filter(
        self, event_filter: dict[MultiharpDeviceIndex, MultiharpMainEventFilter]
    ) -> None:
        pass

    async def stream_measurement(
        self, measurement_time: Quantity, output_queue: asyncio.Queue[RawRecords]
    ) -> None:
        measurement_time_seconds = measurement_time.to(mhtk_ureg.seconds).magnitude
        start_time = time.perf_counter()
        while (time.perf_counter() - start_time) < measurement_time_seconds:
            await output_queue.put(self.__generate_raw_records())
            await asyncio.sleep(0.01)
        output_queue.shutdown()

    def __generate_raw_records(self) -> RawRecords:
        raw_records: RawRecords = []
        for event_time in range(500000):
            raw_records.append(0x02000001 + event_time)
        return raw_records
