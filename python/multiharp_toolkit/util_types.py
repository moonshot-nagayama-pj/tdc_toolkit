from typing import TypeAlias, TypedDict, cast

import multiharp_toolkit._mhtk_rs as mh
import pyarrow as pa
from pyarrow import DataType, RecordBatch

T2WRAPAROUND_V1 = 33552000
T2WRAPAROUND_V2 = 33554432

Channel: TypeAlias = int
"""channel number. usually sync ch is 0."""

TimeTag: TypeAlias = float
"""time in ps"""

RawMeasData: TypeAlias = int

TimeTagDataSchema = pa.schema(
    [("ch", cast(DataType, pa.int8())), ("timestamp", cast(DataType, pa.float64()))]
)


class DeviceInputChannelConfig(TypedDict):
    edge_trigger_level: int
    edge_trigger: "mh.Edge"
    channel_offset: int
    enable: bool


class DeviceConfig(TypedDict):
    sync_divider: int
    sync_edge_trigger_level: int  # mV
    sync_edge: "mh.Edge"
    sync_channel_offset: int  # ps
    sync_channel_enable: bool
    inputs: list[DeviceInputChannelConfig]


class MeasStartMarker:
    config: DeviceConfig
    measurement_duration: float
    param: dict[str, str | int | float]

    def __init__(
        self,
        config: DeviceConfig,
        measurement_duration: float,
        param: dict[str, str | int | float] | None = None,
    ) -> None:
        self.config = config
        self.measurement_duration = measurement_duration
        if param is None:
            param = {}
        self.param = param


class MeasEndMarker:
    pass


RawMeasDataSequence: TypeAlias = list[RawMeasData | MeasEndMarker | MeasStartMarker]
ParsedMeasDataSequence: TypeAlias = RecordBatch | MeasEndMarker | MeasStartMarker
