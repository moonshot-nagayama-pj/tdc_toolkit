import os, asyncio
from argparse import ArgumentParser
from multiharp_toolkit.stream_parser import StreamParser
from concurrent.futures import ThreadPoolExecutor
import pyarrow as pa
import pyarrow.compute as pc
import polars as pl
import plotly.express as px
import pyarrow.parquet as pq

import multiharp_toolkit._mhtk_rs as mh
from multiharp_toolkit.coincidence_counter import ChannelInfo, CoincidenceCounter
from multiharp_toolkit.device import list_device_index, Device
from multiharp_toolkit.histogram import Histogram
from multiharp_toolkit.ptu_parser import parse
from multiharp_toolkit.util_types import Channel, TimeTagDataSchema, DeviceConfig


def measure():
    dev_ids = list_device_index()
    if not dev_ids:
        print("no device")
        exit(0)
    print("available devices: ", dev_ids)
    config: DeviceConfig = {
        "sync_channel_offset": 0,
        "sync_divider": 1,
        "sync_edge": mh.Edge.Falling,
        "sync_edge_trigger_level": -70,
        "sync_channel_enable": True,
        "inputs": [
            {
                "enable": True,
                "channel_offset": 0,
                "edge_trigger": mh.Edge.Falling,
                "edge_trigger_level": -70,
            }
        ]
        * 16,
        # TODO: Need to include "globRes" for StreamParser
    }

    dev = Device(dev_ids[0], config)
    parser = StreamParser(dev.queue)

    async def run():
        with dev.open():
            with ThreadPoolExecutor(max_workers=4) as e:
                print("start measurement")
                main_loop = asyncio.get_event_loop()
                main_loop.run_in_executor(e, dev.start_measurement, 1000)
                parser_task = asyncio.create_task(parser.run())
                await asyncio.gather(parser_task)
        table = pa.ipc.open_file(parser.filename).read_all()
        fname = os.path.basename(parser.filename).replace(".arrow", ".parquet")
        pq.write_table(table, f".parquet/{fname}")

    asyncio.run(run())


def ptu2arrow():
    parser = ArgumentParser(
        description="parse .ptu file and save .arrow file under .arrows folder"
    )
    parser.add_argument("inputfile", type=str, help=".ptu file path")
    parser.add_argument("--parquet", type=bool, help="save with parquet format")

    args = parser.parse_args()
    ptu_file_path = args.inputfile
    if not ptu_file_path.endswith(".ptu"):
        print("specify .ptu file")
        exit(1)
    if not os.path.exists(ptu_file_path):
        print("ptu file does not exist")
        exit(1)

    arrow_file_path = os.path.join(
        ".arrows", os.path.basename(ptu_file_path).replace(".ptu", ".arrow")
    )

    if not args.parquet and os.path.exists(arrow_file_path):
        print("arrow file is alread exist:", arrow_file_path)
        exit(0)

    if not os.path.exists(".arrows"):
        os.mkdir(".arrows")

    if not os.path.exists(".parquet"):
        os.mkdir(".parquet")
    print(".ptu file: ", ptu_file_path)
    print(".arrow file path: ", arrow_file_path)

    with open(ptu_file_path, "rb") as f:
        print("load ptu file")
        data = parse(f)
        if data is None:
            print("failed to parse ptu file")
            exit(1)
    ch_arr = []
    ev_arr = []
    for i in range(0, 65):
        ch_arr += [i] * len(data.events[i])
        ev_arr += data.events[i]

    table = pa.table({"ch": ch_arr, "timestamp": ev_arr}, schema=TimeTagDataSchema)

    si = pc.sort_indices(table, sort_keys=[("timestamp", "ascending")])  # type: ignore
    print("\nwrite...", arrow_file_path)
    if args.parquet:
        parquet_file_path = os.path.join(
            ".parquet", os.path.basename(ptu_file_path).replace(".ptu", ".parquet")
        )
        pq.write_table(table.take(si), parquet_file_path)
    else:
        batches = table.take(si).to_batches(max_chunksize=100000)
        with pa.OSFile(arrow_file_path, mode="w") as f:
            with pa.output_stream(arrow_file_path) as f:
                f.writable()
                with pa.ipc.new_file(f, TimeTagDataSchema) as writer:
                    for batch in batches:
                        writer.write_batch(batch)
                f.seekable()


def histogram():
    parser = ArgumentParser(description="read .arrow file and generate histogram")
    parser.add_argument("inputfile", type=str, help="path to the .arrow file")
    parser.add_argument("--sync-ch", type=int, help="sync channel")
    parser.add_argument(
        "--out",
        type=str,
        help="output file path and format. png and html are available",
    )
    parser.add_argument(
        "--channels",
        type=lambda x: list(map(int, x.split(","))),
        help="specify channels. for example: --channels 2,3,4",
    )
    args = parser.parse_args()
    if not args.out.endswith(".png") and not args.out.endswith(".html"):
        print("--out option must end with .png or .html")
        exit(1)
    hist = Histogram(
        base_ch=Channel(args.sync_ch), channels=[Channel(i) for i in args.channels]
    )

    print(pa.total_allocated_bytes(), "bytes")
    print("start processing...")
    df = pl.read_ipc(args.inputfile)
    hist.process(df)

    print(hist.df)
    print(hist)
    print("write...", args.out)
    fig = px.scatter(hist.df, x="bin", y=[f"count_{i}" for i in args.channels])
    fig.update_layout(bargap=0.2)
    if args.out.endswith(".png"):
        fig.write_image(args.out)
    if args.out.endswith(".html"):
        fig.write_html(args.out)


def coincidence():
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
