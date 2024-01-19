from typing import TypeAlias


Channel: TypeAlias = int
"""channel number. usually sync ch is 0."""

TimeTag: TypeAlias = float
"""time in ps"""


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

    @property
    def is_ready(self) -> bool:
        return self.peak_start > 0 and self.peak_end > 0


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
    ):
        self.base_ch = base_ch
        self.channels = [base_ch] + target_channels
        self.length = len(self.channels)
        self.i = 0
        self.count = 0
        self.name = str([c.ch for c in self.channels])
        self.base_start = 0
        self.last_truetime = 0

    def process(self, ch: Channel, truetime: TimeTag):
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

    @property
    def is_ready(self) -> bool:
        for c in self.channels[1:]:
            if not c.is_ready:
                return False
        return True


class HistogramState:
    timediffs: dict[Channel, list[TimeTag]]
    channels: list[Channel]

    # state
    base_ch: Channel
    base_start: TimeTag
    last_truetime: TimeTag

    def __init__(self, base_ch: Channel, channels: list[Channel]) -> None:
        self.base_ch = base_ch
        self.channels = channels
        self.base_start = 0
        self.last_truetime = 0
        self.timediffs = {}
        for ch in self.channels:
            self.timediffs[ch] = []

    def process(self, ch: Channel, truetime: TimeTag):
        self.last_truetime = truetime
        if ch == self.base_ch:
            self.base_start = truetime
        elif ch in self.timediffs:
            self.timediffs[ch].append(truetime - self.base_start)


class CoincidenceCounter:
    number_of_counts: dict[Channel, int]  # key, int
    peak_windows: dict[Channel, tuple[TimeTag, TimeTag]]  # (peak start, peak end) in ps

    histograms: list[HistogramState]
    coincidence_counters: list[CoincidenceCounterState]

    def __init__(
        self,
        histogram_targets: list[Channel] = [],  # the first element must be base channel
        coincidence_targets: list[list[ChannelInfo | Channel]] = [],
    ):
        self.number_of_counts = {}
        self.histograms = []
        self.peak_windows = dict()
        self.coincidence_counters = []
        target_channels: list[Channel] = []

        if len(histogram_targets) > 0:
            assert (
                len(set(histogram_targets)) >= 2
            ), "must specify histogram targets at least 2 channels"
            base_ch, *channels = histogram_targets
            self.histograms.append(HistogramState(base_ch, channels))
            target_channels += histogram_targets

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

    def process_events(self, events: list[tuple[Channel, TimeTag]]):
        for ev in events:
            self.process(*ev)

    def process(self, ch: Channel, truetime: TimeTag):
        if ch not in self.number_of_counts:
            return
        self.number_of_counts[ch] += 1
        for h in self.histograms:
            h.process(ch, truetime)
        for c in self.coincidence_counters:
            if c.is_ready:
                c.process(ch, truetime)

    def setup_peakwindows(self):
        for c in self.coincidence_counters:
            for i, ch_info in enumerate(c.channels):
                if i == 0:  # this is base channel
                    continue
                if not ch_info.is_ready:
                    ch = ch_info.ch
                    if ch not in self.peak_windows:
                        raise RuntimeError(
                            f"ch({ch}) does not have a specific peak window."
                        )
                    peak_start, peak_end = self.peak_windows[ch]
                    ch_info.peak_start = peak_start
                    ch_info.peak_end = peak_end

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
