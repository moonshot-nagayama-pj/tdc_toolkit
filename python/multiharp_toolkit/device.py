import asyncio
import time
from dataclasses import dataclass, field

import multiharp_toolkit._mhtk_rs as mh
import structlog
from multiharp_toolkit.exceptions import (
    FifoOverrunException,
    InvalidStateException,
    MeasurementCompletedException,
)
from multiharp_toolkit.interface import RawRecords
from pint import Quantity

log = structlog.get_logger()


@dataclass(frozen=True, kw_only=True)
class DeviceInputChannelConfig:
    edge_trigger_level: int
    edge_trigger: "mh.Edge"
    channel_offset: int
    enable: bool


@dataclass(frozen=True, kw_only=True)
class DeviceConfig:
    sync_divider: int
    sync_edge_trigger_level: int  # mV
    sync_edge: "mh.Edge"
    sync_channel_offset: int  # ps
    sync_channel_enable: bool
    inputs: list[DeviceInputChannelConfig]


@dataclass(frozen=True, kw_only=True)
class Device:
    device_index: int
    config: DeviceConfig
    # Track whether this object is "closed," meaning that it is
    # uninitialized, or "open," meaning that the MultiHarp has been
    # initialized and is ready to make measurements. This object can
    # only be used with a single configuration. To reconfigure the
    # device, create a new object.
    closed: bool = field(default=False, init=False)

    # Track whether this object is "closed," meaning that it is
    # uninitialized, or "open," meaning that the MultiHarp has been
    # initialized and is ready to make measurements. This object can
    # only be used with a single configuration. To reconfigure the
    # device, create a new object.
    configured: bool = field(default=False, init=False)

    def open(self) -> None:
        if self.configured or self.closed:
            raise InvalidStateException(
                f"The device was already configured or closed and should not be re-opened. Configured: {self.configured} Closed: {self.closed}"
            )

        mh.open_device(self.device_index)
        mh.initialize(self.device_index, mh.Mode.T2, mh.RefSource.InternalClock)
        self.__configure()

        # Sample code notes that "after Init or SetSyncDiv you must allow >100 ms for valid count rate readings"
        time.sleep(0.2)

        object.__setattr__(self, "configured", True)

    def close(self) -> None:
        try:
            mh.close_device(self.device_index)
        finally:
            object.__setattr__(self, "closed", True)

    def __configure(self) -> None:
        c = self.config
        dev_id = self.device_index

        num_inputs = mh.get_number_of_input_channels(dev_id)
        num_configured_inputs = len(self.config.inputs)
        if num_inputs != num_configured_inputs:
            raise InvalidStateException(
                f"The number of configured inputs must match the actual number of inputs. Configured inputs: {num_configured_inputs} Actual inputs: {num_inputs}"
            )

        mh.set_sync_divider(dev_id, c.sync_divider)
        mh.set_sync_edge_trigger(dev_id, c.sync_edge_trigger_level, c.sync_edge)
        mh.set_sync_channel_offset(dev_id, 0)
        mh.set_sync_channel_enable(dev_id, c.sync_channel_enable)
        for ch in range(0, num_inputs):
            ch_config = c.inputs[ch]
            mh.set_input_edge_trigger(
                dev_id, ch, ch_config.edge_trigger_level, ch_config.edge_trigger
            )
            mh.set_input_channel_offset(dev_id, ch, ch_config.channel_offset)
            mh.set_input_channel_enable(dev_id, ch, ch_config.enable)

    async def stream_measurement(
        self, measurement_time: Quantity, output_queue: asyncio.Queue[RawRecords]
    ) -> None:
        measurement_time_ms = measurement_time.to("milliseconds").magnitude
        try:
            mh.start_measurement(self.device_index, measurement_time_ms)
            while True:
                self.__check_fifo_overrun()
                raw_records = self.__read_fifo()
                if raw_records is not None:
                    await output_queue.put(raw_records)
        finally:
            output_queue.shutdown()
            mh.stop_measurement(self.device_index)

    def __check_fifo_overrun(self) -> None:
        flags = mh.get_flags(self.device_index)
        if flags & 2:  # FLAG_FIFOFULL
            log.error(event="FIFO overrun")
            raise FifoOverrunException()

    def __read_fifo(self) -> RawRecords | None:
        # https://github.com/pylint-dev/pylint/issues/9354
        # pylint: disable-next=unpacking-non-sequence
        record_count, raw_data = mh.read_fifo(self.device_index)
        if record_count > 0:
            return RawRecords(raw_data=raw_data, record_count=record_count)
        if mh.ctc_status(self.device_index) > 0:
            raise MeasurementCompletedException()
        return None


def list_device_index() -> list[int]:
    available_devices = []
    for i in range(0, 8):
        try:
            mh.open_device(i)
            available_devices.append(i)
        # pylint: disable-next=broad-exception-caught,unused-variable
        except Exception as e:  # noqa: F841
            pass
        finally:
            mh.close_device(i)
    return available_devices
