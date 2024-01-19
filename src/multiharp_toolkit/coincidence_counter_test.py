import pytest
from .coincidence_counter import (
    CoincidenceCounter,
    ChannelInfo,
    CoincidenceCounterState,
)


def test_init():
    counter = CoincidenceCounter(
        histogram_targets=[0, 1, 2], coincidence_targets=[[0, 1], [0, 2], [0, 1, 2]]
    )
    assert len(counter.histograms) == 1


def test_process_simple_count():
    counter = CoincidenceCounter(histogram_targets=[0, 1, 2])
    counter.process(0, 10.0)
    assert counter.number_of_counts == {0: 1, 1: 0, 2: 0}

    counter.process(0, 20.0)
    assert counter.number_of_counts == {0: 2, 1: 0, 2: 0}

    counter.process(1, 21.0)
    assert counter.number_of_counts == {0: 2, 1: 1, 2: 0}

    counter.process(2, 22.0)
    assert counter.number_of_counts == {0: 2, 1: 1, 2: 1}

    counter.process(2, 23.0)
    assert counter.number_of_counts == {0: 2, 1: 1, 2: 2}


def test_process_ignore_unknown_ch():
    counter = CoincidenceCounter(histogram_targets=[0, 1, 2])

    assert counter.number_of_counts == {0: 0, 1: 0, 2: 0}
    counter.process(10, 10.0)
    assert counter.number_of_counts == {0: 0, 1: 0, 2: 0}


def test_process_timediff():
    counter = CoincidenceCounter(histogram_targets=[0, 1, 2])
    events = [(0, 10.0), (1, 20.0), (0, 30.0), (1, 40.0), (2, 45.0), (0, 50.0)]
    counter.process_events(events)
    assert counter.number_of_counts == {0: 3, 1: 2, 2: 1}
    assert counter.histograms[0].timediffs == {1: [10.0, 10.0], 2: [15.0]}


def test_count_coincidence():
    counter = CoincidenceCounter(
        coincidence_targets=[
            [ChannelInfo(0), ChannelInfo(1, 9.0, 11.0), ChannelInfo(2, 14.0, 16.0)]
        ]
    )
    events = [(0, 10.0), (1, 20.0), (0, 30.0), (1, 40.0), (2, 45.0), (0, 50.0)]
    counter.process_events(events)

    assert counter.coincidence_counters[0].name == "[0, 1, 2]"
    assert counter.coincidence_counters[0].count == 1
    assert counter.coincidence_counts == {"[0, 1, 2]": 1}


def test_ch_info():
    info = ChannelInfo(1, peak_start=0, peak_end=1)
    assert info.in_peak_window(0.9)
    assert not info.in_peak_window(1.0)
    assert not info.in_peak_window(1.1)


def test_one_coincidencounter():
    cc = CoincidenceCounterState(
        ChannelInfo(0), [ChannelInfo(1, 1.0, 2.0), ChannelInfo(2, 3.0, 4.0)]
    )
    assert cc.i == 0
    cc.process(0, 10.0)
    assert cc.base_start == 10.0
    assert cc.i == 1
    cc.process(1, 11.5)
    assert cc.base_start == 10.0
    assert cc.i == 2
    cc.process(2, 13.5)
    assert cc.i == 0
    assert cc.base_start == 10.0
    assert cc.count == 1

    cc.process(0, 24.0)
    assert cc.base_start == 24.0
    assert cc.i == 1
    cc.process(0, 30.0)
    assert cc.base_start == 30.0
    assert cc.i == 1
