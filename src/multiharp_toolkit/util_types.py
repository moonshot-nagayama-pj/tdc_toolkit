from typing import TypeAlias
import pyarrow as pa

Channel: TypeAlias = int
"""channel number. usually sync ch is 0."""

TimeTag: TypeAlias = float
"""time in ps"""


TimeTagDataSchema = pa.schema([("ch", pa.int8()), ("timestamp", pa.float64())])
