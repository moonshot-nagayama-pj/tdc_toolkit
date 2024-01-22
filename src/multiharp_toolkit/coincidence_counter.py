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


class CoincidenceCounter:
    number_of_counts: dict[Channel, int]  # key, int
    peak_windows: dict[Channel, tuple[TimeTag, TimeTag]]  # (peak start, peak end) in ps

    coincidence_counters: list[CoincidenceCounterState]

    def __init__(
        self,
        coincidence_targets: list[list[ChannelInfo | Channel]]
        | list[list[ChannelInfo]] = [],
    ):
        self.number_of_counts = {}
        self.peak_windows = dict()
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

    def process_arrow(self, arrow_file_path):
        data: pa.RecordBatchFileReader = pa.ipc.open_file(arrow_file_path)
        for i in range(0, data.num_record_batches):
            batch = data.get_batch(i)
            channels = batch["ch"].tolist()
            timestamps = batch["timestamp"].tolist()
            for i, ch in enumerate(channels):
                timestamp = timestamps[i]
                self.process(ch, timestamp)

    def process_events(self, events: list[tuple[Channel, TimeTag]]):
        for ev in events:
            self.process(*ev)

    def process(self, ch: Channel, truetime: TimeTag):
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


if __name__ == "__main__":
    import pyarrow as pa
    from argparse import ArgumentParser

    parser = ArgumentParser(description="count coincidence from arrow file")
    parser.add_argument("inputfile", type=str, help="path to the .arrow file")
    parser.add_argument(
        "--sync-ch", type=int, help="sync channel", default=0, required=False
    )

    def parse_channel_and_peak(value):
        v = value.split(",")
        return [int(v[0]), float(v[1]), float(v[2])]

    parser.add_argument(
        "--channel",
        nargs="+",
        type=parse_channel_and_peak,
        help="specify channel and peak window in ps: --channel [ch],[peak_start],[peak_end]. for example: --channel 1,500,600 2,700,800",
    )

    args = parser.parse_args()
    print(args)
    sync_ch = args.sync_ch
    channels = args.channel
    counter = CoincidenceCounter(
        [
            [ChannelInfo(sync_ch)]
            + [ChannelInfo(*channels[i]) for i in range(0, len(channels))]
        ]
    )
    counter.process_arrow(args.inputfile)
    print("coincidence counts:")
    for k, v in counter.coincidence_counts.items():
        print(k, v)
    print("\nch | count")
    for k, v in counter.number_of_counts.items():
        print(k, "  ", v)
