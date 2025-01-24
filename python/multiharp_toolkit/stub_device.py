import asyncio
from dataclasses import dataclass

from multiharp_toolkit.interface import (
    MultiharpDevice,
    MultiharpDeviceChannel,
    MultiharpDeviceIndex,
    MultiharpDeviceInfo,
    MultiharpMainEventFilter,
    RawRecords,
)
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
        for _ in range(1, 10):
            await output_queue.put(
                RawRecords(
                    raw_data=[0x02000001, 0x02000002, 0x02000003], record_count=3
                )
            )
        output_queue.shutdown()
