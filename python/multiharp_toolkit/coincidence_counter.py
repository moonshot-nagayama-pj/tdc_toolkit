from typing import cast

import pyarrow as pa
from multiharp_toolkit.util_types import Channel, TimeTag


class ChannelInfo:
    ch: Channel
    peak_start: TimeTag
    peak_end: TimeTag

    def __init__(
        self, ch: Channel, peak_start: TimeTag = 0, peak_end: TimeTag = 0
    ) -> None:
        self.ch = ch
        self.peak_start = peak_start
        self.peak_end = peak_end

    def in_peak_window(self, truetime: TimeTag) -> bool:
        return self.peak_start < truetime < self.peak_end


class CoincidenceCounterState:
    # config
    name: str
    base_ch: ChannelInfo
    length: int
    channels: list[ChannelInfo]

    # state
    i: int
    base_start: TimeTag
    last_truetime: TimeTag
    count: int

    def __init__(
        self,
        base_ch: ChannelInfo,
        target_channels: list[ChannelInfo],
    ) -> None:
        self.base_ch = base_ch
        self.channels = [base_ch] + target_channels
        self.length = len(self.channels)
        self.i = 0
        self.count = 0
        self.name = str([c.ch for c in self.channels])
        self.base_start = 0
        self.last_truetime = 0

    def process(self, ch: Channel, truetime: TimeTag) -> None:
        self.last_truetime = truetime
        if ch == self.base_ch.ch:
            self.base_start = truetime
            self.i = 1
            return

        ch_info = self.channels[self.i]
        if ch == ch_info.ch:
            diff = truetime - self.base_start
            if ch_info.in_peak_window(diff):
                self.i += 1

        if self.i == self.length:
            self.count += 1
            self.i = 0

    def __str__(self) -> str:
        return f"CCState({self.base_ch},{self.channels}, count:{self.count})"


class CoincidenceCounter:
    number_of_counts: dict[Channel, int]  # key, int
    peak_windows: dict[Channel, tuple[TimeTag, TimeTag]]  # (peak start, peak end) in ps

    coincidence_counters: list[CoincidenceCounterState]

    def __init__(
        self,
        coincidence_targets: (
            list[list[ChannelInfo | Channel]] | list[list[ChannelInfo]]
        ),
    ) -> None:
        self.number_of_counts = {}
        self.peak_windows = {}
        self.coincidence_counters = []
        target_channels: list[Channel] = []

        for target in coincidence_targets:
            assert (
                len(set(target)) >= 2
            ), "must specify coincnidence targets at least 2 channels"
            base_ch, *channels = [
                ch if isinstance(ch, ChannelInfo) else ChannelInfo(ch) for ch in target
            ]
            self.coincidence_counters.append(CoincidenceCounterState(base_ch, channels))
            target_channels += map(lambda x: x.ch, [base_ch] + channels)

        for ch in set(target_channels):
            self.number_of_counts[ch] = 0

    def process_arrow(self, arrow_file_path: str) -> None:
        with open(arrow_file_path, "rb") as arrow_file:
            data: pa.RecordBatchFileReader = pa.ipc.open_file(arrow_file, options=None)
            for i in range(0, data.num_record_batches):
                batch = data.get_batch(i)
                channels = cast(list[Channel], batch["ch"].tolist())
                timestamps = cast(list[TimeTag], batch["timestamp"].tolist())
                for i, ch in enumerate(channels):
                    timestamp = timestamps[i]
                    self.process(ch, timestamp)

    def process_events(self, events: list[tuple[Channel, TimeTag]]) -> None:
        for ev in events:
            self.process(*ev)

    def process(self, ch: Channel, truetime: TimeTag) -> None:
        if ch not in self.number_of_counts:
            return
        self.number_of_counts[ch] += 1
        for c in self.coincidence_counters:
            c.process(ch, truetime)

    @property
    def coincidence_counts(self) -> dict[str, int]:
        tmp = {}
        for counter in self.coincidence_counters:
            tmp[counter.name] = counter.count
        return tmp


def extract(lst: list[int | list[int]]) -> tuple[list[int], list[list[int]]]:
    s: set[int] = set()
    coincidence_tuples: list[list[int]] = []
    for e in lst:
        if isinstance(e, int):
            s.add(e)
        else:
            for i in e:
                s.add(i)
            coincidence_tuples.append(e)
    return (list(s), coincidence_tuples)
