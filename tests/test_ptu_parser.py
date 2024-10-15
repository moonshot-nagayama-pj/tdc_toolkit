from multiharp_toolkit import ptu_parser
import pickle
import pytest

EXAMPLE_PTU_FILES = [
    "sampledata/example-10mins-ch1-t2",
    "sampledata/example-1min-ch1-t2",
]


@pytest.mark.parametrize("filename", EXAMPLE_PTU_FILES)
@pytest.mark.xfail
def test_parse_ptu_file(filename: str) -> None:
    with open(f"{filename}.ptu", "rb") as f:
        result = ptu_parser.parse(f)
    assert result is not None
    assert result.globRes == 5e-12
    assert len(result.names) == len(result.values)
    assert len(result.events) == 65  # event contains 64 ch + sync

    with open(f"{filename}.pickle", "rb") as f:
        expected_events = pickle.load(f)

    for ch in range(0, 65):
        assert len(result.events[ch]) == len(expected_events[ch])
        for i in range(len(result.events[ch])):
            assert (
                result.events[ch][i] == expected_events[ch][i]
            ), f"faled at ch: {ch}, index: {i}"
